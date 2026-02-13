#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_adequacy_evidence.sh [options]

Options:
  --selection <path>     Selection config file (default: models/system/adequacy_selection.env)
  --output-dir <dir>     Output directory (default: .ci-artifacts/adequacy-evidence)
  --output <path>        Markdown report output path
  --json <path>          JSON output path
  --status-file <path>   Status file output path (PASS/OPEN)
  --enforce-pass         Exit non-zero when adequacy status is OPEN
  -h, --help             Show this help
USAGE
}

selection_file="${REPO_ROOT}/models/system/adequacy_selection.env"
output_dir="${REPO_ROOT}/.ci-artifacts/adequacy-evidence"
output_file=""
json_file=""
status_file=""
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --selection)
      selection_file="$2"
      shift 2
      ;;
    --output-dir)
      output_dir="$2"
      shift 2
      ;;
    --output)
      output_file="$2"
      shift 2
      ;;
    --json)
      json_file="$2"
      shift 2
      ;;
    --status-file)
      status_file="$2"
      shift 2
      ;;
    --enforce-pass)
      enforce_pass=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ ! -f "${selection_file}" ]]; then
  echo "Selection file not found: ${selection_file}" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${selection_file}"

: "${ADEQUACY_CLASS_ID:?missing ADEQUACY_CLASS_ID in selection file}"
: "${ADEQUACY_CLASS_NAME:?missing ADEQUACY_CLASS_NAME in selection file}"
: "${PASS_FIXTURE:?missing PASS_FIXTURE in selection file}"
: "${FAIL_FIXTURE:?missing FAIL_FIXTURE in selection file}"

pass_fixture_path="${REPO_ROOT}/${PASS_FIXTURE}"
fail_fixture_path="${REPO_ROOT}/${FAIL_FIXTURE}"

if [[ ! -f "${pass_fixture_path}" ]]; then
  echo "Pass fixture not found: ${pass_fixture_path}" >&2
  exit 1
fi
if [[ ! -f "${fail_fixture_path}" ]]; then
  echo "Fail fixture not found: ${fail_fixture_path}" >&2
  exit 1
fi

mkdir -p "${output_dir}"
output_file="${output_file:-${output_dir}/adequacy-differential.md}"
json_file="${json_file:-${output_dir}/adequacy-evidence.json}"
status_file="${status_file:-${output_dir}/adequacy.status}"

declare -a records=()

evaluate_fixture() {
  local label="$1"
  local model_path="$2"
  local expected="$3"

  local rust_verdict="ERROR"
  local formal_verdict="ERROR"
  local formal_solver_status="OPEN"
  local category=""
  local expected_match="false"
  local expected_solver="SAT"
  local model_rel="${model_path#${REPO_ROOT}/}"
  local fixture_dir="${output_dir}/solver/${label}"
  local fixture_expectations_file="${fixture_dir}/expectations.tsv"
  local fixture_solver_status_file="${fixture_dir}/alloy-solver.status"
  local fixture_solver_report_file="${fixture_dir}/alloy-solver.md"
  local fixture_solver_json_file="${fixture_dir}/alloy-solver.json"

  if concern_output="$(cd -- "${REPO_ROOT}" && cargo run -p pf_dsl -- "${model_path}" --concern-coverage 2>&1)"; then
    rust_verdict="$(
      printf '%s\n' "${concern_output}" \
        | grep -E "^- Concern coverage status: " \
        | sed -e 's/^- Concern coverage status: //'
    )"
    rust_verdict="${rust_verdict:-ERROR}"
  fi

  if [[ "${expected}" == "FAIL" ]]; then
    expected_solver="UNSAT"
  fi

  mkdir -p "${fixture_dir}"
  cat > "${fixture_expectations_file}" <<EOF
# model_pattern|command_pattern|expected|note
${model_rel}|*|${expected_solver}|Selected adequacy expectation for ${label}
*|*|SAT|Default expectation for non-selected models
EOF

  if bash "${REPO_ROOT}/scripts/run_alloy_solver_check.sh" \
    --model "${model_path}" \
    --output-dir "${fixture_dir}" \
    --report "${fixture_solver_report_file}" \
    --json "${fixture_solver_json_file}" \
    --status-file "${fixture_solver_status_file}" \
    --expectations "${fixture_expectations_file}" >/dev/null 2>&1; then
    formal_solver_status="$(cat "${fixture_solver_status_file}" 2>/dev/null || true)"
    if [[ "${formal_solver_status}" == "PASS" ]]; then
      formal_verdict="PASS"
    else
      formal_verdict="FAIL"
    fi
  fi

  if [[ "${rust_verdict}" == "${formal_verdict}" ]]; then
    case "${rust_verdict}" in
      PASS) category="both_pass" ;;
      FAIL) category="both_fail" ;;
      *) category="both_error" ;;
    esac
  elif [[ "${rust_verdict}" == "PASS" ]]; then
    category="rust_only_pass"
  elif [[ "${formal_verdict}" == "PASS" ]]; then
    category="formal_only_pass"
  else
    category="mixed_non_pass"
  fi

  if [[ "${rust_verdict}" == "${expected}" && "${formal_verdict}" == "${expected}" ]]; then
    expected_match="true"
  fi

  records+=("${label}|${model_path}|${expected}|${expected_solver}|${rust_verdict}|${formal_verdict}|${formal_solver_status}|${category}|${expected_match}")
}

evaluate_fixture "expected_pass" "${pass_fixture_path}" "PASS"
evaluate_fixture "expected_fail" "${fail_fixture_path}" "FAIL"

mismatch_count=0
for record in "${records[@]}"; do
  IFS='|' read -r _ _ _ _ _ _ _ _ expected_match <<< "${record}"
  if [[ "${expected_match}" != "true" ]]; then
    mismatch_count=$((mismatch_count + 1))
  fi
done

overall_status="PASS"
if [[ "${mismatch_count}" -ne 0 ]]; then
  overall_status="OPEN"
fi

{
  echo "# Adequacy Differential Report"
  echo
  echo "- Selected class ID: \`${ADEQUACY_CLASS_ID}\`"
  echo "- Selected class name: \`${ADEQUACY_CLASS_NAME}\`"
  echo "- Status: \`${overall_status}\`"
  echo "- Mismatches: ${mismatch_count}"
  echo
  echo "## Fixture Verdicts"
  echo
  echo "| Fixture | Expected | Expected Solver | Rust Verdict | Formal Verdict | Solver Status | Category | Match |"
  echo "| --- | --- | --- | --- | --- | --- | --- | --- |"
  for record in "${records[@]}"; do
    IFS='|' read -r label model expected expected_solver rust formal formal_solver_status category expected_match <<< "${record}"
    echo "| ${label} (\`${model#${REPO_ROOT}/}\`) | ${expected} | ${expected_solver} | ${rust} | ${formal} | ${formal_solver_status} | ${category} | ${expected_match} |"
  done
} > "${output_file}"

{
  echo "{"
  echo "  \"class_id\": \"${ADEQUACY_CLASS_ID}\","
  echo "  \"class_name\": \"${ADEQUACY_CLASS_NAME}\","
  echo "  \"status\": \"${overall_status}\","
  echo "  \"mismatches\": ${mismatch_count},"
  echo "  \"fixtures\": ["
  for index in "${!records[@]}"; do
    IFS='|' read -r label model expected expected_solver rust formal formal_solver_status category expected_match <<< "${records[$index]}"
    comma=","
    if [[ "${index}" -eq "$((${#records[@]} - 1))" ]]; then
      comma=""
    fi
    echo "    {"
    echo "      \"label\": \"${label}\","
    echo "      \"model\": \"${model#${REPO_ROOT}/}\","
    echo "      \"expected\": \"${expected}\","
    echo "      \"expected_solver\": \"${expected_solver}\","
    echo "      \"rust_verdict\": \"${rust}\","
    echo "      \"formal_verdict\": \"${formal}\","
    echo "      \"formal_solver_status\": \"${formal_solver_status}\","
    echo "      \"category\": \"${category}\","
    echo "      \"match\": ${expected_match}"
    echo "    }${comma}"
  done
  echo "  ]"
  echo "}"
} > "${json_file}"

echo "${overall_status}" > "${status_file}"

echo "Generated ${output_file} (status: ${overall_status})"
echo "Generated ${json_file}"

if [[ "${overall_status}" != "PASS" && "${enforce_pass}" -eq 1 ]]; then
  exit 1
fi
