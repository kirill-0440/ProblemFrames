#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_adequacy_evidence.sh [options]

Options:
  --selection <path>      Selection config file (default: models/system/adequacy_selection.env)
  --expectations <path>   Command-level expectation manifest for solver checks
  --output-dir <dir>      Output directory (default: .ci-artifacts/adequacy-evidence)
  --output <path>         Markdown report output path
  --json <path>           JSON output path
  --status-file <path>    Status file output path (PASS/OPEN)
  --closure-matrix-tsv <path>
                         Aggregated obligation closure matrix (TSV)
  --closure-matrix-md <path>
                         Aggregated obligation closure matrix (Markdown)
  --enforce-pass          Exit non-zero when adequacy status is OPEN
  -h, --help              Show this help
USAGE
}

selection_file="${REPO_ROOT}/models/system/adequacy_selection.env"
expectations_file=""
output_dir="${REPO_ROOT}/.ci-artifacts/adequacy-evidence"
output_file=""
json_file=""
status_file=""
closure_matrix_tsv=""
closure_matrix_md=""
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --selection)
      selection_file="$2"
      shift 2
      ;;
    --expectations)
      expectations_file="$2"
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
    --closure-matrix-tsv)
      closure_matrix_tsv="$2"
      shift 2
      ;;
    --closure-matrix-md)
      closure_matrix_md="$2"
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

if [[ -z "${expectations_file}" ]]; then
  expectations_file="${ADEQUACY_EXPECTATIONS:-models/system/adequacy_expectations.tsv}"
fi
if [[ "${expectations_file}" != /* ]]; then
  expectations_file="${REPO_ROOT}/${expectations_file}"
fi

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
if [[ ! -f "${expectations_file}" ]]; then
  echo "Adequacy expectations file not found: ${expectations_file}" >&2
  exit 1
fi

mkdir -p "${output_dir}"
output_file="${output_file:-${output_dir}/adequacy-differential.md}"
json_file="${json_file:-${output_dir}/adequacy-evidence.json}"
status_file="${status_file:-${output_dir}/adequacy.status}"
closure_matrix_tsv="${closure_matrix_tsv:-${output_dir}/adequacy-obligation-closure.tsv}"
closure_matrix_md="${closure_matrix_md:-${output_dir}/adequacy-obligation-closure.md}"

if [[ "${closure_matrix_tsv}" != /* ]]; then
  closure_matrix_tsv="${REPO_ROOT}/${closure_matrix_tsv}"
fi
if [[ "${closure_matrix_md}" != /* ]]; then
  closure_matrix_md="${REPO_ROOT}/${closure_matrix_md}"
fi

mkdir -p "$(dirname -- "${closure_matrix_tsv}")"
mkdir -p "$(dirname -- "${closure_matrix_md}")"

declare -a records=()
declare -a closure_rows=()

json_number_field_or_default() {
  local json_path="$1"
  local field_name="$2"
  local fallback="$3"
  local value

  value="$(
    grep -E "\"${field_name}\":" "${json_path}" 2>/dev/null \
      | head -n 1 \
      | sed -E "s/.*\"${field_name}\":[[:space:]]*([0-9]+).*/\\1/" || true
  )"

  if [[ "${value}" =~ ^[0-9]+$ ]]; then
    printf '%s' "${value}"
  else
    printf '%s' "${fallback}"
  fi
}

json_string_field_or_default() {
  local json_path="$1"
  local field_name="$2"
  local fallback="$3"
  local value

  value="$(
    grep -E "\"${field_name}\":" "${json_path}" 2>/dev/null \
      | head -n 1 \
      | sed -E "s/.*\"${field_name}\":[[:space:]]*\"([^\"]*)\".*/\\1/" || true
  )"

  if [[ -n "${value}" ]]; then
    printf '%s' "${value}"
  else
    printf '%s' "${fallback}"
  fi
}

evaluate_fixture() {
  local label="$1"
  local model_path="$2"
  local expected="$3"

  local rust_verdict="ERROR"
  local formal_verdict="ERROR"
  local formal_solver_status="OPEN"
  local category=""
  local expected_match="false"
  local model_rel="${model_path#${REPO_ROOT}/}"
  local fixture_dir="${output_dir}/solver/${label}"
  local fixture_solver_status_file="${fixture_dir}/alloy-solver.status"
  local fixture_solver_report_file="${fixture_dir}/alloy-solver.md"
  local fixture_solver_json_file="${fixture_dir}/alloy-solver.json"
  local fixture_solver_matrix_tsv_file="${fixture_dir}/alloy-solver-command-closure.tsv"
  local fixture_solver_matrix_md_file="${fixture_dir}/alloy-solver-command-closure.md"

  local solver_required_total=0
  local solver_required_missing=0
  local solver_mismatch_count=0
  local solver_command_count=0
  local solver_missing_rules=""

  if concern_output="$(cd -- "${REPO_ROOT}" && cargo run -p pf_dsl -- "${model_path}" --concern-coverage 2>&1)"; then
    rust_verdict="$(
      printf '%s\n' "${concern_output}" \
        | grep -E "^- Concern coverage status: " \
        | sed -e 's/^- Concern coverage status: //'
    )"
    rust_verdict="${rust_verdict:-ERROR}"
  fi

  mkdir -p "${fixture_dir}"
  if bash "${REPO_ROOT}/scripts/run_alloy_solver_check.sh" \
    --model "${model_path}" \
    --output-dir "${fixture_dir}" \
    --report "${fixture_solver_report_file}" \
    --json "${fixture_solver_json_file}" \
    --status-file "${fixture_solver_status_file}" \
    --closure-matrix-tsv "${fixture_solver_matrix_tsv_file}" \
    --closure-matrix-md "${fixture_solver_matrix_md_file}" \
    --expectations "${expectations_file}" >/dev/null 2>&1; then
    formal_solver_status="$(cat "${fixture_solver_status_file}" 2>/dev/null || true)"
    formal_solver_status="${formal_solver_status:-OPEN}"
  else
    formal_solver_status="$(cat "${fixture_solver_status_file}" 2>/dev/null || true)"
    formal_solver_status="${formal_solver_status:-OPEN}"
  fi

  if [[ -f "${fixture_solver_json_file}" ]]; then
    solver_required_total="$(json_number_field_or_default "${fixture_solver_json_file}" "required_expectation_rules" "0")"
    solver_required_missing="$(json_number_field_or_default "${fixture_solver_json_file}" "required_expectation_rules_missing" "0")"
    solver_mismatch_count="$(json_number_field_or_default "${fixture_solver_json_file}" "expectation_mismatch_count" "0")"
    solver_command_count="$(json_number_field_or_default "${fixture_solver_json_file}" "command_count" "0")"
    solver_missing_rules="$(json_string_field_or_default "${fixture_solver_json_file}" "required_expectation_missing_rules" "")"
  fi

  if [[ "${formal_solver_status}" == "PASS" ]]; then
    formal_verdict="PASS"
  else
    formal_verdict="FAIL"
  fi

  if [[ -f "${fixture_solver_matrix_tsv_file}" ]]; then
    while IFS='|' read -r _model_entry command_name expected_verdict actual_verdict command_status rule_id required_flag note; do
      if [[ -z "${_model_entry}" || "${_model_entry}" == \#* ]]; then
        continue
      fi
      closure_rows+=(
        "${label}|${model_rel}|${command_name}|${expected_verdict}|${actual_verdict}|${command_status}|${rule_id}|${required_flag}|${note}"
      )
    done < "${fixture_solver_matrix_tsv_file}"
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

  records+=("${label}|${model_path}|${expected}|${rust_verdict}|${formal_verdict}|${formal_solver_status}|${solver_required_total}|${solver_required_missing}|${solver_mismatch_count}|${solver_command_count}|${solver_missing_rules}|${category}|${expected_match}")
}

evaluate_fixture "expected_pass" "${pass_fixture_path}" "PASS"
evaluate_fixture "expected_fail" "${fail_fixture_path}" "FAIL"

mismatch_count=0
for record in "${records[@]}"; do
  IFS='|' read -r _ _ _ _ _ _ _ _ _ _ _ _ expected_match <<< "${record}"
  if [[ "${expected_match}" != "true" ]]; then
    mismatch_count=$((mismatch_count + 1))
  fi
done

overall_status="PASS"
if [[ "${mismatch_count}" -ne 0 ]]; then
  overall_status="OPEN"
fi

{
  echo "# fixture|model|command|expected|actual|status|rule|required|note"
  for closure_row in "${closure_rows[@]}"; do
    echo "${closure_row}"
  done
} > "${closure_matrix_tsv}"

{
  echo "# Adequacy Obligation Closure Matrix"
  echo
  echo "- Selected class ID: \`${ADEQUACY_CLASS_ID}\`"
  echo "- Selected class name: \`${ADEQUACY_CLASS_NAME}\`"
  echo "- Status: \`${overall_status}\`"
  echo
  echo "| Fixture | Model | Command | Expected | Actual | Status | Rule | Required | Note |"
  echo "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"
  if [[ "${#closure_rows[@]}" -eq 0 ]]; then
    echo "| - | - | - | - | - | no_commands | - | - | - |"
  else
    for closure_row in "${closure_rows[@]}"; do
      IFS='|' read -r fixture_label fixture_model command_name expected_verdict actual_verdict command_status rule_id required_flag note <<< "${closure_row}"
      required_text="optional"
      if [[ "${required_flag}" == "1" ]]; then
        required_text="required"
      fi
      if [[ -z "${note}" ]]; then
        note="-"
      fi
      echo "| ${fixture_label} | \`${fixture_model}\` | \`${command_name}\` | ${expected_verdict} | ${actual_verdict} | ${command_status} | \`${rule_id}\` | ${required_text} | ${note} |"
    done
  fi
} > "${closure_matrix_md}"

{
  echo "# Adequacy Differential Report"
  echo
  echo "- Selected class ID: \`${ADEQUACY_CLASS_ID}\`"
  echo "- Selected class name: \`${ADEQUACY_CLASS_NAME}\`"
  echo "- Expectations manifest: \`${expectations_file#${REPO_ROOT}/}\`"
  echo "- Obligation closure matrix: \`${closure_matrix_tsv#${REPO_ROOT}/}\`"
  echo "- Status: \`${overall_status}\`"
  echo "- Mismatches: ${mismatch_count}"
  echo
  echo "## Fixture Verdicts"
  echo
  echo "| Fixture | Expected | Rust Verdict | Formal Verdict | Solver Status | Required Rules | Missing Required | Mismatches | Commands | Missing Rule IDs | Category | Match |"
  echo "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
  for record in "${records[@]}"; do
    IFS='|' read -r label model expected rust formal formal_solver_status required_total required_missing solver_mismatches solver_commands missing_rules category expected_match <<< "${record}"
    if [[ -z "${missing_rules}" ]]; then
      missing_rules="-"
    fi
    echo "| ${label} (\`${model#${REPO_ROOT}/}\`) | ${expected} | ${rust} | ${formal} | ${formal_solver_status} | ${required_total} | ${required_missing} | ${solver_mismatches} | ${solver_commands} | ${missing_rules} | ${category} | ${expected_match} |"
  done
} > "${output_file}"

{
  echo "{"
  echo "  \"class_id\": \"${ADEQUACY_CLASS_ID}\","
  echo "  \"class_name\": \"${ADEQUACY_CLASS_NAME}\","
  echo "  \"expectations_manifest\": \"${expectations_file#${REPO_ROOT}/}\","
  echo "  \"obligation_closure_matrix_tsv\": \"${closure_matrix_tsv#${REPO_ROOT}/}\","
  echo "  \"obligation_closure_matrix_md\": \"${closure_matrix_md#${REPO_ROOT}/}\","
  echo "  \"status\": \"${overall_status}\","
  echo "  \"mismatches\": ${mismatch_count},"
  echo "  \"fixtures\": ["
  for index in "${!records[@]}"; do
    IFS='|' read -r label model expected rust formal formal_solver_status required_total required_missing solver_mismatches solver_commands missing_rules category expected_match <<< "${records[$index]}"
    comma=","
    if [[ "${index}" -eq "$((${#records[@]} - 1))" ]]; then
      comma=""
    fi
    echo "    {"
    echo "      \"label\": \"${label}\","
    echo "      \"model\": \"${model#${REPO_ROOT}/}\","
    echo "      \"expected\": \"${expected}\","
    echo "      \"rust_verdict\": \"${rust}\","
    echo "      \"formal_verdict\": \"${formal}\","
    echo "      \"formal_solver_status\": \"${formal_solver_status}\","
    echo "      \"required_expectation_rules\": ${required_total},"
    echo "      \"required_expectation_rules_missing\": ${required_missing},"
    echo "      \"expectation_mismatches\": ${solver_mismatches},"
    echo "      \"command_count\": ${solver_commands},"
    echo "      \"missing_required_rule_ids\": \"${missing_rules}\","
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
echo "Generated ${closure_matrix_tsv}"
echo "Generated ${closure_matrix_md}"

if [[ "${overall_status}" != "PASS" && "${enforce_pass}" -eq 1 ]]; then
  exit 1
fi
