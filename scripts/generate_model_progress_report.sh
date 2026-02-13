#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/generate_model_progress_report.sh [options]

Options:
  --model <path>                    PF model path (default: models/system/tool_spec.pf)
  --output-dir <dir>                Output directory (default: .ci-artifacts/model-progress)
  --output <path>                   Markdown report output path
  --json <path>                     JSON output path
  --status-file <path>              Status output path (PASS/OPEN)
  --implementation-trace-md <path>  Existing implementation-trace markdown artifact
  --implementation-trace-status <path>
                                    Existing implementation-trace status artifact
  --adequacy-closure-tsv <path>     Existing adequacy closure matrix artifact
  --adequacy-status <path>          Existing adequacy status artifact
  --policy <path>                   Policy file for implementation trace generation
                                    (default: models/system/implementation_trace_policy.env)
  --selection <path>                Adequacy selection env file
                                    (default: models/system/adequacy_selection.env)
  --expectations <path>             Adequacy expectations file (generated when omitted)
  --skip-generate-inputs            Require provided artifacts; do not generate missing inputs
  --enforce-pass                    Exit non-zero when status is not PASS
  -h, --help                        Show this help
USAGE
}

model_path="${REPO_ROOT}/models/system/tool_spec.pf"
output_dir="${REPO_ROOT}/.ci-artifacts/model-progress"
output_file=""
json_file=""
status_file=""
impl_trace_md_file=""
impl_trace_status_file=""
adequacy_closure_tsv_file=""
adequacy_status_file=""
policy_file="${REPO_ROOT}/models/system/implementation_trace_policy.env"
selection_file="${REPO_ROOT}/models/system/adequacy_selection.env"
expectations_file=""
generate_inputs=1
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      model_path="$2"
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
    --implementation-trace-md)
      impl_trace_md_file="$2"
      shift 2
      ;;
    --implementation-trace-status)
      impl_trace_status_file="$2"
      shift 2
      ;;
    --adequacy-closure-tsv)
      adequacy_closure_tsv_file="$2"
      shift 2
      ;;
    --adequacy-status)
      adequacy_status_file="$2"
      shift 2
      ;;
    --policy)
      policy_file="$2"
      shift 2
      ;;
    --selection)
      selection_file="$2"
      shift 2
      ;;
    --expectations)
      expectations_file="$2"
      shift 2
      ;;
    --skip-generate-inputs)
      generate_inputs=0
      shift
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

if [[ "${model_path}" != /* ]]; then
  model_path="${REPO_ROOT}/${model_path}"
fi
if [[ "${output_dir}" != /* ]]; then
  output_dir="${REPO_ROOT}/${output_dir}"
fi
if [[ -n "${output_file}" && "${output_file}" != /* ]]; then
  output_file="${REPO_ROOT}/${output_file}"
fi
if [[ -n "${json_file}" && "${json_file}" != /* ]]; then
  json_file="${REPO_ROOT}/${json_file}"
fi
if [[ -n "${status_file}" && "${status_file}" != /* ]]; then
  status_file="${REPO_ROOT}/${status_file}"
fi
if [[ -n "${impl_trace_md_file}" && "${impl_trace_md_file}" != /* ]]; then
  impl_trace_md_file="${REPO_ROOT}/${impl_trace_md_file}"
fi
if [[ -n "${impl_trace_status_file}" && "${impl_trace_status_file}" != /* ]]; then
  impl_trace_status_file="${REPO_ROOT}/${impl_trace_status_file}"
fi
if [[ -n "${adequacy_closure_tsv_file}" && "${adequacy_closure_tsv_file}" != /* ]]; then
  adequacy_closure_tsv_file="${REPO_ROOT}/${adequacy_closure_tsv_file}"
fi
if [[ -n "${adequacy_status_file}" && "${adequacy_status_file}" != /* ]]; then
  adequacy_status_file="${REPO_ROOT}/${adequacy_status_file}"
fi
if [[ "${policy_file}" != /* ]]; then
  policy_file="${REPO_ROOT}/${policy_file}"
fi
if [[ "${selection_file}" != /* ]]; then
  selection_file="${REPO_ROOT}/${selection_file}"
fi
if [[ -n "${expectations_file}" && "${expectations_file}" != /* ]]; then
  expectations_file="${REPO_ROOT}/${expectations_file}"
fi

if [[ ! -f "${model_path}" ]]; then
  echo "Model file not found: ${model_path}" >&2
  exit 1
fi
if [[ ! -f "${selection_file}" ]]; then
  echo "Selection file not found: ${selection_file}" >&2
  exit 1
fi

mkdir -p "${output_dir}"
output_file="${output_file:-${output_dir}/progress.md}"
json_file="${json_file:-${output_dir}/progress.json}"
status_file="${status_file:-${output_dir}/progress.status}"
impl_trace_md_file="${impl_trace_md_file:-${output_dir}/implementation-trace.md}"
impl_trace_status_file="${impl_trace_status_file:-${output_dir}/implementation-trace.status}"
adequacy_closure_tsv_file="${adequacy_closure_tsv_file:-${output_dir}/adequacy-obligation-closure.tsv}"
adequacy_status_file="${adequacy_status_file:-${output_dir}/adequacy.status}"

mkdir -p "$(dirname -- "${output_file}")"
mkdir -p "$(dirname -- "${json_file}")"
mkdir -p "$(dirname -- "${status_file}")"
mkdir -p "$(dirname -- "${impl_trace_md_file}")"
mkdir -p "$(dirname -- "${impl_trace_status_file}")"
mkdir -p "$(dirname -- "${adequacy_closure_tsv_file}")"
mkdir -p "$(dirname -- "${adequacy_status_file}")"

if [[ "${generate_inputs}" -eq 1 ]]; then
  if [[ ! -f "${impl_trace_md_file}" || ! -f "${impl_trace_status_file}" ]]; then
    trace_args=(
      --output "${impl_trace_md_file}"
      --status-file "${impl_trace_status_file}"
    )
    if [[ -f "${policy_file}" ]]; then
      trace_args+=(--policy "${policy_file}")
    fi
    bash "${REPO_ROOT}/scripts/check_model_implementation_trace.sh" "${trace_args[@]}" "${model_path}"
  fi

  if [[ ! -f "${adequacy_closure_tsv_file}" || ! -f "${adequacy_status_file}" ]]; then
    if [[ -z "${expectations_file}" ]]; then
      expectations_file="${output_dir}/adequacy_expectations.generated.tsv"
    fi
    bash "${REPO_ROOT}/scripts/generate_adequacy_expectations.sh" \
      --selection "${selection_file}" \
      --output "${expectations_file}"
    bash "${REPO_ROOT}/scripts/run_adequacy_evidence.sh" \
      --selection "${selection_file}" \
      --expectations "${expectations_file}" \
      --output "${output_dir}/adequacy-differential.md" \
      --json "${output_dir}/adequacy-evidence.json" \
      --status-file "${adequacy_status_file}" \
      --closure-matrix-tsv "${adequacy_closure_tsv_file}" \
      --closure-matrix-md "${output_dir}/adequacy-obligation-closure.md"
  fi
fi

if [[ ! -f "${impl_trace_md_file}" ]]; then
  echo "Implementation trace markdown is missing: ${impl_trace_md_file}" >&2
  exit 1
fi
if [[ ! -f "${impl_trace_status_file}" ]]; then
  echo "Implementation trace status is missing: ${impl_trace_status_file}" >&2
  exit 1
fi
if [[ ! -f "${adequacy_closure_tsv_file}" ]]; then
  echo "Adequacy closure matrix is missing: ${adequacy_closure_tsv_file}" >&2
  exit 1
fi
if [[ ! -f "${adequacy_status_file}" ]]; then
  echo "Adequacy status is missing: ${adequacy_status_file}" >&2
  exit 1
fi

impl_status="$(cat "${impl_trace_status_file}" 2>/dev/null || true)"
impl_status="${impl_status:-OPEN}"
adequacy_status="$(cat "${adequacy_status_file}" 2>/dev/null || true)"
adequacy_status="${adequacy_status:-OPEN}"

requirements_total="$(
  grep -E "^- Requirements total: " "${impl_trace_md_file}" \
    | head -n 1 \
    | sed -E 's/^- Requirements total: ([0-9]+).*/\1/' || true
)"
implemented_count="$(
  grep -E "^- Implemented: " "${impl_trace_md_file}" \
    | head -n 1 \
    | sed -E 's/^- Implemented: ([0-9]+).*/\1/' || true
)"
partial_count="$(
  grep -E "^- Partial: " "${impl_trace_md_file}" \
    | head -n 1 \
    | sed -E 's/^- Partial: ([0-9]+).*/\1/' || true
)"
planned_count="$(
  grep -E "^- Planned: " "${impl_trace_md_file}" \
    | head -n 1 \
    | sed -E 's/^- Planned: ([0-9]+).*/\1/' || true
)"
policy_status="$(
  grep -E "^- Policy status: " "${impl_trace_md_file}" \
    | head -n 1 \
    | sed -E 's/^- Policy status: (.*)/\1/' || true
)"

requirements_total="${requirements_total:-0}"
implemented_count="${implemented_count:-0}"
partial_count="${partial_count:-0}"
planned_count="${planned_count:-0}"
policy_status="${policy_status:-SKIPPED}"

mapfile -t partial_requirements < <(
  awk -F'|' '
    /^\| / {
      requirement=$2
      status=$4
      gsub(/^ +| +$/, "", requirement)
      gsub(/^ +| +$/, "", status)
      if (requirement == "Requirement" || requirement == "---") next
      if (status == "partial") print requirement
    }
  ' "${impl_trace_md_file}"
)
mapfile -t planned_requirements < <(
  awk -F'|' '
    /^\| / {
      requirement=$2
      status=$4
      gsub(/^ +| +$/, "", requirement)
      gsub(/^ +| +$/, "", status)
      if (requirement == "Requirement" || requirement == "---") next
      if (status == "planned") print requirement
    }
  ' "${impl_trace_md_file}"
)

command_total=0
command_match=0
command_mismatch=0
required_total=0
required_match=0
required_mismatch=0
declare -a mismatch_rows=()

while IFS='|' read -r fixture_label model_label command_name expected_verdict actual_verdict command_status rule_label required_flag note_text; do
  if [[ -z "${fixture_label}" || "${fixture_label}" == \#* ]]; then
    continue
  fi
  command_total=$((command_total + 1))
  required_numeric=0
  case "${required_flag}" in
    1|required|true|yes) required_numeric=1 ;;
  esac
  if [[ "${required_numeric}" -eq 1 ]]; then
    required_total=$((required_total + 1))
  fi

  if [[ "${command_status}" == "MATCH" ]]; then
    command_match=$((command_match + 1))
    if [[ "${required_numeric}" -eq 1 ]]; then
      required_match=$((required_match + 1))
    fi
  else
    command_mismatch=$((command_mismatch + 1))
    if [[ "${required_numeric}" -eq 1 ]]; then
      required_mismatch=$((required_mismatch + 1))
    fi
    mismatch_rows+=("${fixture_label}|${command_name}|${expected_verdict}|${actual_verdict}|${required_numeric}|${rule_label}")
  fi
done < "${adequacy_closure_tsv_file}"

overall_status="PASS"
declare -a status_notes=()
if [[ "${impl_status}" != "PASS" ]]; then
  overall_status="OPEN"
  status_notes+=("implementation trace status is ${impl_status}")
fi
if [[ "${adequacy_status}" != "PASS" ]]; then
  overall_status="OPEN"
  status_notes+=("adequacy status is ${adequacy_status}")
fi
if [[ "${required_mismatch}" -gt 0 ]]; then
  status_notes+=("required obligation mismatches observed: ${required_mismatch} (interpreted via adequacy_status)")
fi

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  value="${value//$'\r'/\\r}"
  value="${value//$'\t'/\\t}"
  printf '%s' "${value}"
}

{
  echo "# Model Progress Report"
  echo
  echo "- Model: \`${model_path#${REPO_ROOT}/}\`"
  echo "- Status: \`${overall_status}\`"
  echo "- Implementation trace status: \`${impl_status}\`"
  echo "- Adequacy status: \`${adequacy_status}\`"
  echo "- Policy status: \`${policy_status}\`"
  echo "- Requirements: total=${requirements_total}, implemented=${implemented_count}, partial=${partial_count}, planned=${planned_count}"
  echo "- Obligation commands: total=${command_total}, match=${command_match}, mismatch=${command_mismatch}"
  echo "- Required obligation commands: total=${required_total}, match=${required_match}, mismatch=${required_mismatch}"
  if [[ "${#status_notes[@]}" -gt 0 ]]; then
    echo "- Status notes: $(IFS='; '; echo "${status_notes[*]}")"
  fi
  echo
  echo "## Remaining Requirements"
  echo
  echo "### Partial"
  if [[ "${#partial_requirements[@]}" -eq 0 ]]; then
    echo "- None."
  else
    for requirement in "${partial_requirements[@]}"; do
      echo "- ${requirement}"
    done
  fi
  echo
  echo "### Planned"
  if [[ "${#planned_requirements[@]}" -eq 0 ]]; then
    echo "- None."
  else
    for requirement in "${planned_requirements[@]}"; do
      echo "- ${requirement}"
    done
  fi
  echo
  echo "## Obligation Mismatches"
  echo
  if [[ "${#mismatch_rows[@]}" -eq 0 ]]; then
    echo "- None."
  else
    echo "| Fixture | Command | Expected | Actual | Required | Rule |"
    echo "| --- | --- | --- | --- | --- | --- |"
    for mismatch_row in "${mismatch_rows[@]}"; do
      IFS='|' read -r fixture_label command_name expected_verdict actual_verdict required_numeric rule_label <<< "${mismatch_row}"
      required_text="optional"
      if [[ "${required_numeric}" -eq 1 ]]; then
        required_text="required"
      fi
      echo "| ${fixture_label} | \`${command_name}\` | ${expected_verdict} | ${actual_verdict} | ${required_text} | \`${rule_label}\` |"
    done
  fi
  echo
  echo "## Inputs"
  echo
  echo "- Implementation trace markdown: \`${impl_trace_md_file#${REPO_ROOT}/}\`"
  echo "- Implementation trace status: \`${impl_trace_status_file#${REPO_ROOT}/}\`"
  echo "- Adequacy closure matrix: \`${adequacy_closure_tsv_file#${REPO_ROOT}/}\`"
  echo "- Adequacy status: \`${adequacy_status_file#${REPO_ROOT}/}\`"
} > "${output_file}"

{
  echo "{"
  echo "  \"model\": \"$(json_escape "${model_path#${REPO_ROOT}/}")\","
  echo "  \"status\": \"${overall_status}\","
  echo "  \"implementation_trace_status\": \"${impl_status}\","
  echo "  \"adequacy_status\": \"${adequacy_status}\","
  echo "  \"policy_status\": \"$(json_escape "${policy_status}")\","
  echo "  \"requirements\": {"
  echo "    \"total\": ${requirements_total},"
  echo "    \"implemented\": ${implemented_count},"
  echo "    \"partial\": ${partial_count},"
  echo "    \"planned\": ${planned_count}"
  echo "  },"
  echo "  \"obligations\": {"
  echo "    \"total\": ${command_total},"
  echo "    \"match\": ${command_match},"
  echo "    \"mismatch\": ${command_mismatch},"
  echo "    \"required_total\": ${required_total},"
  echo "    \"required_match\": ${required_match},"
  echo "    \"required_mismatch\": ${required_mismatch}"
  echo "  },"
  echo "  \"status_notes\": ["
  for index in "${!status_notes[@]}"; do
    comma=","
    if [[ "${index}" -eq "$((${#status_notes[@]} - 1))" ]]; then
      comma=""
    fi
    echo "    \"$(json_escape "${status_notes[$index]}")\"${comma}"
  done
  echo "  ],"
  echo "  \"partial_requirements\": ["
  for index in "${!partial_requirements[@]}"; do
    comma=","
    if [[ "${index}" -eq "$((${#partial_requirements[@]} - 1))" ]]; then
      comma=""
    fi
    echo "    \"$(json_escape "${partial_requirements[$index]}")\"${comma}"
  done
  echo "  ],"
  echo "  \"planned_requirements\": ["
  for index in "${!planned_requirements[@]}"; do
    comma=","
    if [[ "${index}" -eq "$((${#planned_requirements[@]} - 1))" ]]; then
      comma=""
    fi
    echo "    \"$(json_escape "${planned_requirements[$index]}")\"${comma}"
  done
  echo "  ],"
  echo "  \"inputs\": {"
  echo "    \"implementation_trace_markdown\": \"$(json_escape "${impl_trace_md_file#${REPO_ROOT}/}")\","
  echo "    \"implementation_trace_status\": \"$(json_escape "${impl_trace_status_file#${REPO_ROOT}/}")\","
  echo "    \"adequacy_closure_tsv\": \"$(json_escape "${adequacy_closure_tsv_file#${REPO_ROOT}/}")\","
  echo "    \"adequacy_status\": \"$(json_escape "${adequacy_status_file#${REPO_ROOT}/}")\""
  echo "  }"
  echo "}"
} > "${json_file}"

echo "${overall_status}" > "${status_file}"

echo "Generated ${output_file} (status: ${overall_status})"
echo "Generated ${json_file}"

if [[ "${enforce_pass}" -eq 1 && "${overall_status}" != "PASS" ]]; then
  exit 1
fi
