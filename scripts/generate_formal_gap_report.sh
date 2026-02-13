#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/generate_formal_gap_report.sh [options]

Options:
  --model <path>               PF model path (default: models/system/tool_spec.pf)
  --closure-rows-tsv <path>    requirement formal-closure rows TSV (default: .ci-artifacts/system-model/tool_spec.formal-closure.rows.tsv)
  --traceability-csv <path>    traceability CSV (default: .ci-artifacts/system-model/tool_spec.traceability.csv)
  --output <path>              markdown report output (default: .ci-artifacts/system-model/tool_spec.formal-gap.md)
  --json <path>                JSON report output (default: .ci-artifacts/system-model/tool_spec.formal-gap.json)
  --status-file <path>         status output file (default: .ci-artifacts/system-model/tool_spec.formal-gap.status)
  --enforce-pass               exit non-zero when status is not PASS
  -h, --help                   show this help
USAGE
}

model_file="${REPO_ROOT}/models/system/tool_spec.pf"
closure_rows_tsv="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-closure.rows.tsv"
traceability_csv="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.traceability.csv"
output_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-gap.md"
json_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-gap.json"
status_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-gap.status"
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      model_file="$2"
      shift 2
      ;;
    --closure-rows-tsv)
      closure_rows_tsv="$2"
      shift 2
      ;;
    --traceability-csv)
      traceability_csv="$2"
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

for path_var in model_file closure_rows_tsv traceability_csv output_file json_file status_file; do
  current_path="${!path_var}"
  if [[ "${current_path}" != /* ]]; then
    printf -v "${path_var}" '%s/%s' "${REPO_ROOT}" "${current_path}"
  fi
done

for input in "${model_file}" "${closure_rows_tsv}" "${traceability_csv}"; do
  if [[ ! -f "${input}" ]]; then
    echo "Required file not found: ${input}" >&2
    exit 1
  fi
done

mkdir -p "$(dirname -- "${output_file}")"
mkdir -p "$(dirname -- "${json_file}")"
mkdir -p "$(dirname -- "${status_file}")"

tmp_requirements_tsv="$(mktemp)"
tmp_subproblems_tsv="$(mktemp)"
cleanup() {
  rm -f "${tmp_requirements_tsv}" "${tmp_subproblems_tsv}"
}
trap cleanup EXIT

cargo run -p pf_dsl -- "${model_file}" --requirements-tsv > "${tmp_requirements_tsv}"

awk -F',' '
  NR == 1 { next }
  $2 == "subproblem" && $4 == "subproblem_includes_requirement" && $5 == "requirement" {
    req = $6
    subproblem_id = $3
    if (req == "" || subproblem_id == "") {
      next
    }
    if (req in mapping) {
      if (index(", " mapping[req] ", ", ", " subproblem_id ", ") == 0) {
        mapping[req] = mapping[req] ", " subproblem_id
      }
    } else {
      mapping[req] = subproblem_id
    }
  }
  END {
    for (req in mapping) {
      printf "%s|%s\n", req, mapping[req]
    }
  }
' "${traceability_csv}" | LC_ALL=C sort > "${tmp_subproblems_tsv}"

declare -A requirement_frame=()
while IFS='|' read -r requirement frame layer extra; do
  if [[ -z "${requirement}" || "${requirement}" == \#* ]]; then
    continue
  fi
  if [[ -n "${extra:-}" ]]; then
    echo "Invalid requirements TSV row: ${requirement}|${frame}|${layer}|${extra}" >&2
    exit 1
  fi
  if [[ -n "${layer:-}" && "${layer}" != "CIM" && "${layer}" != "PIM" && "${layer}" != "PSM" && "${layer}" != "UNSPECIFIED" ]]; then
    echo "Invalid requirements TSV layer value for ${requirement}: ${layer}" >&2
    exit 1
  fi
  requirement_frame["${requirement}"]="${frame}"
done < "${tmp_requirements_tsv}"

declare -A requirement_status=()
declare -A requirement_reason=()
declare -A requirement_argument=()
while IFS='|' read -r requirement argument status reason extra; do
  if [[ -z "${requirement}" || "${requirement}" == \#* ]]; then
    continue
  fi
  if [[ -n "${extra:-}" ]]; then
    echo "Invalid closure rows TSV row: ${requirement}|${argument}|${status}|${reason}|${extra}" >&2
    exit 1
  fi
  requirement_argument["${requirement}"]="${argument}"
  requirement_status["${requirement}"]="${status}"
  requirement_reason["${requirement}"]="${reason}"
done < "${closure_rows_tsv}"

declare -A requirement_subproblems=()
while IFS='|' read -r requirement subproblems extra; do
  if [[ -z "${requirement}" ]]; then
    continue
  fi
  if [[ -n "${extra:-}" ]]; then
    echo "Invalid subproblem mapping row: ${requirement}|${subproblems}|${extra}" >&2
    exit 1
  fi
  requirement_subproblems["${requirement}"]="${subproblems}"
done < "${tmp_subproblems_tsv}"

mapfile -t sorted_requirements < <(
  printf '%s\n' "${!requirement_frame[@]}" | LC_ALL=C sort
)

rows_tsv_tmp="$(mktemp)"
trap 'cleanup; rm -f "${rows_tsv_tmp}"' EXIT
printf '# requirement|frame|subproblems|argument|closure_status|reason|gap_status\n' > "${rows_tsv_tmp}"

total=0
closed=0
gaps=0

for requirement in "${sorted_requirements[@]}"; do
  frame="${requirement_frame[${requirement}]}"
  subproblems="${requirement_subproblems[${requirement}]:-(none)}"
  argument="${requirement_argument[${requirement}]:-(unmapped)}"
  closure_status="${requirement_status[${requirement}]:-UNKNOWN}"
  reason="${requirement_reason[${requirement}]:-missing_closure_row}"

  gap_status="CLOSED"
  if [[ "${closure_status}" != "CLOSED" ]]; then
    gap_status="GAP"
    gaps=$((gaps + 1))
  else
    closed=$((closed + 1))
  fi

  total=$((total + 1))
  printf '%s|%s|%s|%s|%s|%s|%s\n' \
    "${requirement}" "${frame}" "${subproblems}" "${argument}" "${closure_status}" "${reason}" "${gap_status}" \
    >> "${rows_tsv_tmp}"
done

status="PASS"
if [[ "${gaps}" -gt 0 ]]; then
  status="OPEN"
fi

{
  echo "# Formal Gap Report"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Model source: \`${model_file#${REPO_ROOT}/}\`"
  echo "- Closure rows source: \`${closure_rows_tsv#${REPO_ROOT}/}\`"
  echo "- Traceability source: \`${traceability_csv#${REPO_ROOT}/}\`"
  echo "- Status: \`${status}\`"
  echo
  echo "| Requirement | Frame | Subproblems | Argument | Closure status | Reason | Gap |"
  echo "| --- | --- | --- | --- | --- | --- | --- |"
  awk -F'|' 'NR > 1 { printf("| %s | %s | %s | %s | %s | %s | %s |\n", $1, $2, $3, $4, $5, $6, $7) }' "${rows_tsv_tmp}"
  echo
  echo "- Total requirements: ${total}"
  echo "- Closed: ${closed}"
  echo "- Gaps: ${gaps}"
} > "${output_file}"

json_rows() {
  local first=1
  printf '['
  while IFS='|' read -r requirement frame subproblems argument closure_status reason gap_status; do
    if [[ -z "${requirement}" || "${requirement}" == \#* ]]; then
      continue
    fi
    if [[ "${first}" -eq 0 ]]; then
      printf ', '
    fi
    first=0
    printf '{"requirement":"%s","frame":"%s","subproblems":"%s","argument":"%s","closure_status":"%s","reason":"%s","gap_status":"%s"}' \
      "${requirement}" "${frame}" "${subproblems}" "${argument}" "${closure_status}" "${reason}" "${gap_status}"
  done < "${rows_tsv_tmp}"
  printf ']'
}

{
  echo "{"
  echo "  \"status\": \"${status}\","
  echo "  \"total_requirements\": ${total},"
  echo "  \"closed_count\": ${closed},"
  echo "  \"gap_count\": ${gaps},"
  echo "  \"rows\": $(json_rows)"
  echo "}"
} > "${json_file}"

echo "${status}" > "${status_file}"

echo "Generated ${output_file}"
echo "Generated ${json_file}"

if [[ "${enforce_pass}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Formal gap status is ${status}; expected PASS." >&2
  exit 1
fi
