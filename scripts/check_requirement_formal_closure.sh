#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/check_requirement_formal_closure.sh [options]

Options:
  --model <path>              PF model path (default: models/system/tool_spec.pf)
  --requirements-file <path>  optional explicit requirements source file
  --arguments-file <path>     optional explicit correctness-arguments source file
  --map-file <path>           optional explicit requirement->argument map TSV (default: generated from model)
  --lean-coverage-json <path> Lean coverage JSON (default: .ci-artifacts/system-model/tool_spec.lean-coverage.json)
  --output <path>             markdown report output (default: .ci-artifacts/system-model/tool_spec.formal-closure.md)
  --json <path>               JSON summary output (default: .ci-artifacts/system-model/tool_spec.formal-closure.json)
  --status-file <path>        status file output (default: .ci-artifacts/system-model/tool_spec.formal-closure.status)
  --rows-tsv <path>           per-requirement rows TSV output (default: .ci-artifacts/system-model/tool_spec.formal-closure.rows.tsv)
  --enforce-pass              exit non-zero when status is not PASS
  -h, --help                  show this help
USAGE
}

model_file="${REPO_ROOT}/models/system/tool_spec.pf"
requirements_file=""
arguments_file=""
map_file=""
lean_coverage_json="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.lean-coverage.json"
output_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-closure.md"
json_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-closure.json"
status_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-closure.status"
rows_tsv_file="${REPO_ROOT}/.ci-artifacts/system-model/tool_spec.formal-closure.rows.tsv"
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      model_file="$2"
      shift 2
      ;;
    --requirements-file)
      requirements_file="$2"
      shift 2
      ;;
    --arguments-file)
      arguments_file="$2"
      shift 2
      ;;
    --map-file)
      map_file="$2"
      shift 2
      ;;
    --lean-coverage-json)
      lean_coverage_json="$2"
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
    --rows-tsv)
      rows_tsv_file="$2"
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

for path_var in model_file lean_coverage_json output_file json_file status_file rows_tsv_file; do
  current_path="${!path_var}"
  if [[ "${current_path}" != /* ]]; then
    printf -v "${path_var}" '%s/%s' "${REPO_ROOT}" "${current_path}"
  fi
done

if [[ -n "${requirements_file}" && "${requirements_file}" != /* ]]; then
  requirements_file="${REPO_ROOT}/${requirements_file}"
fi
if [[ -n "${arguments_file}" && "${arguments_file}" != /* ]]; then
  arguments_file="${REPO_ROOT}/${arguments_file}"
fi
if [[ -n "${map_file}" && "${map_file}" != /* ]]; then
  map_file="${REPO_ROOT}/${map_file}"
fi

for input in "${model_file}" "${lean_coverage_json}"; do
  if [[ ! -f "${input}" ]]; then
    echo "Required file not found: ${input}" >&2
    exit 1
  fi
done

mkdir -p "$(dirname -- "${output_file}")"
mkdir -p "$(dirname -- "${json_file}")"
mkdir -p "$(dirname -- "${status_file}")"
mkdir -p "$(dirname -- "${rows_tsv_file}")"

tmp_map_file=""
tmp_requirements_file=""
tmp_arguments_file=""
cleanup() {
  if [[ -n "${tmp_map_file}" && -f "${tmp_map_file}" ]]; then
    rm -f "${tmp_map_file}"
  fi
  if [[ -n "${tmp_requirements_file}" && -f "${tmp_requirements_file}" ]]; then
    rm -f "${tmp_requirements_file}"
  fi
  if [[ -n "${tmp_arguments_file}" && -f "${tmp_arguments_file}" ]]; then
    rm -f "${tmp_arguments_file}"
  fi
}
trap cleanup EXIT

resolved_map_file=""
map_source_label=""
if [[ -n "${map_file}" ]]; then
  if [[ ! -f "${map_file}" ]]; then
    echo "Explicit map file not found: ${map_file}" >&2
    exit 1
  fi
  resolved_map_file="${map_file}"
  map_source_label="${map_file#${REPO_ROOT}/}"
else
  tmp_map_file="$(mktemp)"
  cargo run -p pf_dsl -- "${model_file}" --formal-closure-map-tsv > "${tmp_map_file}"
  resolved_map_file="${tmp_map_file}"
  map_source_label="generated:${model_file#${REPO_ROOT}/}"
fi

resolved_requirements_file=""
requirements_source_label=""
if [[ -n "${requirements_file}" ]]; then
  if [[ ! -f "${requirements_file}" ]]; then
    echo "Requirements file not found: ${requirements_file}" >&2
    exit 1
  fi
  resolved_requirements_file="${requirements_file}"
  requirements_source_label="${requirements_file#${REPO_ROOT}/}"
else
  tmp_requirements_file="$(mktemp)"
  cargo run -p pf_dsl -- "${model_file}" --requirements-tsv > "${tmp_requirements_file}"
  resolved_requirements_file="${tmp_requirements_file}"
  requirements_source_label="generated:${model_file#${REPO_ROOT}/}"
fi

resolved_arguments_file=""
arguments_source_label=""
if [[ -n "${arguments_file}" ]]; then
  if [[ ! -f "${arguments_file}" ]]; then
    echo "Arguments file not found: ${arguments_file}" >&2
    exit 1
  fi
  resolved_arguments_file="${arguments_file}"
  arguments_source_label="${arguments_file#${REPO_ROOT}/}"
else
  tmp_arguments_file="$(mktemp)"
  cargo run -p pf_dsl -- "${model_file}" --correctness-arguments-tsv > "${tmp_arguments_file}"
  resolved_arguments_file="${tmp_arguments_file}"
  arguments_source_label="generated:${model_file#${REPO_ROOT}/}"
fi

if [[ -n "${requirements_file}" ]]; then
  mapfile -t requirements < <(
    grep '^[[:space:]]*requirement "' "${resolved_requirements_file}" \
      | sed -E 's/^[[:space:]]*requirement "([^"]+)".*/\1/'
  )
else
  mapfile -t requirements < <(
    awk -F'|' '
      $0 ~ /^#/ { next }
      NF < 1 { next }
      $1 == "" { next }
      { print $1 }
    ' "${resolved_requirements_file}"
  )
fi

if [[ -n "${arguments_file}" ]]; then
  mapfile -t arguments < <(
    grep '^[[:space:]]*correctnessArgument ' "${resolved_arguments_file}" \
      | sed -E 's/^[[:space:]]*correctnessArgument ([^ ]+) .*/\1/'
  )
else
  mapfile -t arguments < <(
    awk '
      $0 ~ /^#/ { next }
      $0 ~ /^[[:space:]]*$/ { next }
      { gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0); if ($0 != "") print $0 }
    ' "${resolved_arguments_file}"
  )
fi

mapfile -t formalized_arguments < <(
  awk '
    /"formalized": \[/ { in_formalized = 1; next }
    in_formalized && /^\s*],?/ { in_formalized = 0 }
    in_formalized && /"argument":/ {
      line = $0
      sub(/^.*"argument"[[:space:]]*:[[:space:]]*"/, "", line)
      sub(/".*$/, "", line)
      if (line != "") print line
    }
  ' "${lean_coverage_json}"
)

declare -A argument_exists=()
for argument in "${arguments[@]}"; do
  argument_exists["${argument}"]=1
done

declare -A formalized_lookup=()
for argument in "${formalized_arguments[@]}"; do
  formalized_lookup["${argument}"]=1
done

declare -A requirement_to_argument=()
declare -A mapping_duplicates=()

while IFS='|' read -r requirement_id argument_name extra; do
  if [[ -z "${requirement_id}" ]]; then
    continue
  fi
  if [[ "${requirement_id}" == \#* ]]; then
    continue
  fi
  if [[ -n "${extra:-}" ]]; then
    echo "Invalid mapping row (expected two columns): ${requirement_id}|${argument_name}|${extra}" >&2
    exit 1
  fi
  if [[ -z "${argument_name:-}" ]]; then
    echo "Invalid mapping row (missing argument): ${requirement_id}" >&2
    exit 1
  fi
  if [[ -n "${requirement_to_argument[${requirement_id}]:-}" ]]; then
    mapping_duplicates["${requirement_id}"]=1
    continue
  fi
  requirement_to_argument["${requirement_id}"]="${argument_name}"
done < "${resolved_map_file}"

total_requirements="${#requirements[@]}"
closed_count=0
open_count=0
missing_count=0
invalid_argument_count=0
duplicate_mapping_count="${#mapping_duplicates[@]}"

declare -a closed_requirements=()
declare -a open_requirements=()
declare -a missing_requirements=()
declare -a invalid_argument_requirements=()
declare -a closure_rows=()

{
  echo "# Requirement Formal Closure Report"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Model source: \`${model_file#${REPO_ROOT}/}\`"
  echo "- Requirements source: \`${requirements_source_label}\`"
  echo "- Arguments source: \`${arguments_source_label}\`"
  echo "- Mapping source: \`${map_source_label}\`"
  echo "- Lean coverage source: \`${lean_coverage_json#${REPO_ROOT}/}\`"
  echo
  echo "| Requirement | Correctness argument | Status | Reason |"
  echo "| --- | --- | --- | --- |"

  printf '# requirement|correctness_argument|status|reason\n' > "${rows_tsv_file}"

  for requirement in "${requirements[@]}"; do
    mapped_argument="${requirement_to_argument[${requirement}]:-}"
    status=""
    reason=""

    if [[ -z "${mapped_argument}" ]]; then
      status="MISSING_MAP"
      reason="missing_formal_argument_mapping"
      missing_requirements+=("${requirement}")
      missing_count=$((missing_count + 1))
      mapped_argument="(unmapped)"
    elif [[ -z "${argument_exists[${mapped_argument}]:-}" ]]; then
      status="INVALID_ARGUMENT"
      reason="mapped_argument_not_declared"
      invalid_argument_requirements+=("${requirement}")
      invalid_argument_count=$((invalid_argument_count + 1))
    elif [[ -n "${formalized_lookup[${mapped_argument}]:-}" ]]; then
      status="CLOSED"
      reason="formalized"
      closed_requirements+=("${requirement}")
      closed_count=$((closed_count + 1))
    else
      status="OPEN"
      reason="correctness_argument_not_formalized"
      open_requirements+=("${requirement}")
      open_count=$((open_count + 1))
    fi

    echo "| ${requirement} | ${mapped_argument} | ${status} | ${reason} |"
    printf '%s|%s|%s|%s\n' "${requirement}" "${mapped_argument}" "${status}" "${reason}" >> "${rows_tsv_file}"
    closure_rows+=("${requirement}|${mapped_argument}|${status}|${reason}")
  done

  echo
  echo "- Total requirements: ${total_requirements}"
  echo "- Closed: ${closed_count}"
  echo "- Open: ${open_count}"
  echo "- Missing map: ${missing_count}"
  echo "- Invalid argument map: ${invalid_argument_count}"
  echo "- Duplicate map rows: ${duplicate_mapping_count}"
} > "${output_file}"

status="PASS"
if [[ "${open_count}" -gt 0 || "${missing_count}" -gt 0 || "${invalid_argument_count}" -gt 0 || "${duplicate_mapping_count}" -gt 0 ]]; then
  status="OPEN"
fi

json_array_from_list() {
  local -n values_ref="$1"
  local first=1
  printf '['
  for value in "${values_ref[@]}"; do
    if [[ "${first}" -eq 0 ]]; then
      printf ', '
    fi
    first=0
    printf '"%s"' "${value}"
  done
  printf ']'
}

json_rows_from_records() {
  local -n rows_ref="$1"
  local first=1
  printf '['
  for row in "${rows_ref[@]}"; do
    IFS='|' read -r requirement argument row_status reason <<< "${row}"
    if [[ "${first}" -eq 0 ]]; then
      printf ', '
    fi
    first=0
    printf '{"requirement":"%s","correctness_argument":"%s","status":"%s","reason":"%s"}' \
      "${requirement}" "${argument}" "${row_status}" "${reason}"
  done
  printf ']'
}

{
  echo "{"
  echo "  \"status\": \"${status}\","
  echo "  \"total_requirements\": ${total_requirements},"
  echo "  \"closed_count\": ${closed_count},"
  echo "  \"open_count\": ${open_count},"
  echo "  \"missing_map_count\": ${missing_count},"
  echo "  \"invalid_argument_count\": ${invalid_argument_count},"
  echo "  \"duplicate_map_count\": ${duplicate_mapping_count},"
  echo "  \"requirements_source\": \"${requirements_source_label}\","
  echo "  \"arguments_source\": \"${arguments_source_label}\","
  echo "  \"mapping_source\": \"${map_source_label}\","
  echo "  \"rows\": $(json_rows_from_records closure_rows),"
  echo "  \"closed_requirements\": $(json_array_from_list closed_requirements),"
  echo "  \"open_requirements\": $(json_array_from_list open_requirements),"
  echo "  \"missing_requirements\": $(json_array_from_list missing_requirements),"
  echo "  \"invalid_argument_requirements\": $(json_array_from_list invalid_argument_requirements)"
  echo "}"
} > "${json_file}"

echo "${status}" > "${status_file}"

echo "Generated ${output_file}"
echo "Generated ${json_file}"
echo "Generated ${rows_tsv_file}"

if [[ "${enforce_pass}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Requirement formal closure status is ${status}; expected PASS." >&2
  exit 1
fi
