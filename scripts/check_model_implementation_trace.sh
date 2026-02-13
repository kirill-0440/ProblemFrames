#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/check_model_implementation_trace.sh [options] <model.pf>

Options:
  --manifest <path>           Trace manifest file (default: models/system/implementation_trace.tsv)
  --policy <path>             Optional policy env file for staged gating thresholds
  --traceability-csv <path>   Pre-generated traceability CSV (record_type=edge, from_kind=requirement)
  --output <path>             Output markdown file path
  --status-file <path>        Output status file path (contains PASS/OPEN/SKIPPED)
  --policy-status-file <path> Optional policy status output path (PASS/OPEN/SKIPPED)
  --output-dir <dir>          Output directory for derived output paths
  --enforce-pass              Exit non-zero when overall trace status is not PASS
  --enforce-policy            Exit non-zero when policy status is not PASS
  -h, --help                  Show this help.
USAGE
}

manifest_path="${REPO_ROOT}/models/system/implementation_trace.tsv"
policy_path=""
traceability_csv=""
output_path=""
status_path=""
policy_status_path=""
output_dir="${REPO_ROOT}/.ci-artifacts/model-implementation-trace"
enforce_pass=0
enforce_policy=0
model_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --manifest" >&2
        exit 1
      fi
      manifest_path="$2"
      shift 2
      ;;
    --policy)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --policy" >&2
        exit 1
      fi
      policy_path="$2"
      shift 2
      ;;
    --traceability-csv)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --traceability-csv" >&2
        exit 1
      fi
      traceability_csv="$2"
      shift 2
      ;;
    --output)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --output" >&2
        exit 1
      fi
      output_path="$2"
      shift 2
      ;;
    --status-file)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --status-file" >&2
        exit 1
      fi
      status_path="$2"
      shift 2
      ;;
    --policy-status-file)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --policy-status-file" >&2
        exit 1
      fi
      policy_status_path="$2"
      shift 2
      ;;
    --output-dir)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --output-dir" >&2
        exit 1
      fi
      output_dir="$2"
      shift 2
      ;;
    --enforce-pass)
      enforce_pass=1
      shift
      ;;
    --enforce-policy)
      enforce_policy=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
    *)
      if [[ -n "${model_path}" ]]; then
        echo "Model path already provided: ${model_path}" >&2
        exit 1
      fi
      model_path="$1"
      shift
      ;;
  esac
done

if [[ -z "${model_path}" ]]; then
  usage
  exit 1
fi

if [[ ! -f "${model_path}" ]]; then
  echo "Model not found: ${model_path}" >&2
  exit 1
fi

if [[ ! -f "${manifest_path}" ]]; then
  echo "Trace manifest not found: ${manifest_path}" >&2
  exit 1
fi

if [[ "${enforce_policy}" -eq 1 && -z "${policy_path}" ]]; then
  echo "--enforce-policy requires --policy <path>" >&2
  exit 1
fi

abs_model="$(cd -- "$(dirname -- "${model_path}")" && pwd)/$(basename -- "${model_path}")"
if [[ -z "${output_path}" || -z "${status_path}" ]]; then
  mkdir -p "${output_dir}"
  if [[ "${abs_model}" == "${REPO_ROOT}/"* ]]; then
    model_key="${abs_model#${REPO_ROOT}/}"
  else
    model_key="$(basename -- "${abs_model}")"
  fi
  model_key="${model_key%.pf}"
  model_key="${model_key//\//__}"
  if [[ -z "${output_path}" ]]; then
    output_path="${output_dir}/${model_key}.implementation-trace.md"
  fi
  if [[ -z "${status_path}" ]]; then
    status_path="${output_dir}/${model_key}.implementation-trace.status"
  fi
fi

if [[ -n "${policy_path}" && -z "${policy_status_path}" ]]; then
  if [[ "${abs_model}" == "${REPO_ROOT}/"* ]]; then
    model_key="${abs_model#${REPO_ROOT}/}"
  else
    model_key="$(basename -- "${abs_model}")"
  fi
  model_key="${model_key%.pf}"
  model_key="${model_key//\//__}"
  policy_status_path="${output_dir}/${model_key}.implementation-trace.policy.status"
fi

mkdir -p "$(dirname -- "${output_path}")"
mkdir -p "$(dirname -- "${status_path}")"
if [[ -n "${policy_status_path}" ]]; then
  mkdir -p "$(dirname -- "${policy_status_path}")"
fi

resolved_traceability_csv="${traceability_csv}"
cleanup_traceability_csv=0
if [[ -z "${resolved_traceability_csv}" ]]; then
  resolved_traceability_csv="$(mktemp)"
  cleanup_traceability_csv=1
  cargo run -p pf_dsl -- "${model_path}" --traceability-csv > "${resolved_traceability_csv}"
fi

if [[ ! -f "${resolved_traceability_csv}" ]]; then
  echo "Traceability CSV not found: ${resolved_traceability_csv}" >&2
  if [[ "${cleanup_traceability_csv}" -eq 1 ]]; then
    rm -f "${resolved_traceability_csv}"
  fi
  exit 1
fi

mapfile -t requirements < <(
  awk -F, 'NR > 1 && $1 == "edge" && $2 == "requirement" { print $3 }' "${resolved_traceability_csv}" \
    | sort -u
)

if [[ "${cleanup_traceability_csv}" -eq 1 ]]; then
  rm -f "${resolved_traceability_csv}"
fi

if [[ "${#requirements[@]}" -eq 0 ]]; then
  {
    echo "# Implementation Trace Report: $(basename -- "${model_path}")"
    echo
    echo "- Implementation trace status: SKIPPED"
    echo "- Reason: no requirements found in traceability graph."
    echo "- Manifest: \`${manifest_path}\`"
  } > "${output_path}"
  echo "SKIPPED" > "${status_path}"
  echo "Generated ${output_path} (status: SKIPPED)"
  exit 0
fi

declare -A model_requirements=()
declare -A total_checks=()
declare -A passed_checks=()
declare -A details=()
declare -A manifest_requirements=()

for requirement in "${requirements[@]}"; do
  model_requirements["${requirement}"]=1
  total_checks["${requirement}"]=0
  passed_checks["${requirement}"]=0
  details["${requirement}"]=""
done

while IFS='|' read -r requirement_id check_type target value note; do
  line="${requirement_id}${check_type}${target}${value}${note}"
  if [[ -z "${line//[[:space:]]/}" ]]; then
    continue
  fi
  if [[ "${requirement_id}" == \#* ]]; then
    continue
  fi
  if [[ "${requirement_id}" == "requirement_id" ]]; then
    continue
  fi

  manifest_requirements["${requirement_id}"]=1
  if [[ -z "${model_requirements["${requirement_id}"]+x}" ]]; then
    continue
  fi

  total_checks["${requirement_id}"]=$((total_checks["${requirement_id}"] + 1))
  passed=0
  outcome_detail=""

  case "${check_type}" in
    file_contains)
      if [[ -n "${target}" ]]; then
        if [[ "${target}" == /* ]]; then
          resolved_target="${target}"
        else
          resolved_target="${REPO_ROOT}/${target}"
        fi
        if [[ -f "${resolved_target}" ]] && grep -Fq -- "${value}" "${resolved_target}"; then
          passed=1
        fi
      fi
      outcome_detail="file_contains:${target}:${value}"
      ;;
    file_exists)
      if [[ -n "${target}" ]]; then
        if [[ "${target}" == /* ]]; then
          resolved_target="${target}"
        else
          resolved_target="${REPO_ROOT}/${target}"
        fi
        if [[ -e "${resolved_target}" ]]; then
          passed=1
        fi
      fi
      outcome_detail="file_exists:${target}"
      ;;
    command_passes)
      outcome_detail="command_passes:${target}"
      if [[ -n "${target}" ]]; then
        command_log="$(mktemp)"
        # Run in the current shell environment so CI-injected toolchain paths stay visible.
        if (cd -- "${REPO_ROOT}" && bash -c "${target}" >"${command_log}" 2>&1); then
          passed=1
        else
          command_tail="$(tail -n 3 "${command_log}" 2>/dev/null | tr '\n' ';' || true)"
          command_tail="${command_tail%;}"
          if [[ -n "${command_tail}" ]]; then
            outcome_detail="${outcome_detail} [tail: ${command_tail}]"
          fi
        fi
        rm -f "${command_log}"
      fi
      ;;
    manual_pending)
      passed=0
      outcome_detail="manual_pending"
      ;;
    *)
      passed=0
      outcome_detail="unsupported_check_type:${check_type}"
      ;;
  esac

  if [[ "${passed}" -eq 1 ]]; then
    passed_checks["${requirement_id}"]=$((passed_checks["${requirement_id}"] + 1))
    result="PASS"
  else
    result="FAIL"
  fi

  detail_note="${outcome_detail}"
  if [[ -n "${note}" ]]; then
    detail_note="${detail_note} (${note})"
  fi

  if [[ -n "${details["${requirement_id}"]}" ]]; then
    details["${requirement_id}"]+=$'\n'
  fi
  details["${requirement_id}"]+="- ${result}: ${detail_note}"
done < "${manifest_path}"

implemented_count=0
partial_count=0
planned_count=0

remaining_partial=()
remaining_planned=()
rows=()
declare -A requirement_status=()

for requirement in "${requirements[@]}"; do
  total="${total_checks["${requirement}"]}"
  passed="${passed_checks["${requirement}"]}"
  status=""
  note=""

  if [[ "${total}" -eq 0 ]]; then
    status="planned"
    note="no evidence rows mapped in manifest"
    planned_count=$((planned_count + 1))
    remaining_planned+=("${requirement}")
  elif [[ "${passed}" -eq "${total}" ]]; then
    status="implemented"
    note="${passed}/${total} evidence checks passed"
    implemented_count=$((implemented_count + 1))
  elif [[ "${passed}" -gt 0 ]]; then
    status="partial"
    note="${passed}/${total} evidence checks passed"
    partial_count=$((partial_count + 1))
    remaining_partial+=("${requirement}")
  else
    status="planned"
    note="0/${total} evidence checks passed"
    planned_count=$((planned_count + 1))
    remaining_planned+=("${requirement}")
  fi

  rows+=("${requirement}|${passed}/${total}|${status}|${note}")
  requirement_status["${requirement}"]="${status}"
done

overall_status="OPEN"
if [[ "${partial_count}" -eq 0 && "${planned_count}" -eq 0 ]]; then
  overall_status="PASS"
fi

policy_status="SKIPPED"
policy_notes=()
policy_blocking_open_count=0
policy_max_partial="unbounded"
policy_max_planned="unbounded"
policy_max_blocking_open="unbounded"
policy_blocking_prefixes=""

if [[ -n "${policy_path}" ]]; then
  if [[ ! -f "${policy_path}" ]]; then
    echo "Policy file not found: ${policy_path}" >&2
    exit 1
  fi

  # shellcheck disable=SC1090
  source "${policy_path}"

  trace_policy_max_partial="${TRACE_POLICY_MAX_PARTIAL:--1}"
  trace_policy_max_planned="${TRACE_POLICY_MAX_PLANNED:--1}"
  trace_policy_max_blocking_open="${TRACE_POLICY_MAX_BLOCKING_OPEN:--1}"
  trace_policy_blocking_prefixes="${TRACE_POLICY_BLOCKING_PREFIXES:-}"

  policy_max_partial="${trace_policy_max_partial}"
  policy_max_planned="${trace_policy_max_planned}"
  policy_max_blocking_open="${trace_policy_max_blocking_open}"
  policy_blocking_prefixes="${trace_policy_blocking_prefixes}"

  policy_status="PASS"

  if [[ "${trace_policy_max_partial}" =~ ^[0-9]+$ ]] && [[ "${partial_count}" -gt "${trace_policy_max_partial}" ]]; then
    policy_status="OPEN"
    policy_notes+=("partial requirements ${partial_count} exceed policy max ${trace_policy_max_partial}")
  fi

  if [[ "${trace_policy_max_planned}" =~ ^[0-9]+$ ]] && [[ "${planned_count}" -gt "${trace_policy_max_planned}" ]]; then
    policy_status="OPEN"
    policy_notes+=("planned requirements ${planned_count} exceed policy max ${trace_policy_max_planned}")
  fi

  if [[ -n "${trace_policy_blocking_prefixes}" ]]; then
    IFS=',' read -r -a prefixes <<< "${trace_policy_blocking_prefixes}"
    for requirement in "${requirements[@]}"; do
      status="${requirement_status["${requirement}"]}"
      if [[ "${status}" == "implemented" ]]; then
        continue
      fi
      for prefix in "${prefixes[@]}"; do
        trimmed_prefix="${prefix#"${prefix%%[![:space:]]*}"}"
        trimmed_prefix="${trimmed_prefix%"${trimmed_prefix##*[![:space:]]}"}"
        if [[ -n "${trimmed_prefix}" && "${requirement}" == "${trimmed_prefix}"* ]]; then
          policy_blocking_open_count=$((policy_blocking_open_count + 1))
          break
        fi
      done
    done

    if [[ "${trace_policy_max_blocking_open}" =~ ^[0-9]+$ ]] \
      && [[ "${policy_blocking_open_count}" -gt "${trace_policy_max_blocking_open}" ]]; then
      policy_status="OPEN"
      policy_notes+=("blocking-prefix open requirements ${policy_blocking_open_count} exceed policy max ${trace_policy_max_blocking_open}")
    fi
  fi
fi

unknown_manifest_requirements=()
for requirement in "${!manifest_requirements[@]}"; do
  if [[ -z "${model_requirements["${requirement}"]+x}" ]]; then
    unknown_manifest_requirements+=("${requirement}")
  fi
done
if [[ "${#unknown_manifest_requirements[@]}" -gt 0 ]]; then
  IFS=$'\n' unknown_manifest_requirements=($(sort <<<"${unknown_manifest_requirements[*]}"))
  unset IFS
fi

{
  echo "# Implementation Trace Report: $(basename -- "${model_path}")"
  echo
  echo "- Implementation trace status: ${overall_status}"
  echo "- Requirements total: ${#requirements[@]}"
  echo "- Implemented: ${implemented_count}"
  echo "- Partial: ${partial_count}"
  echo "- Planned: ${planned_count}"
  echo "- Manifest: \`${manifest_path}\`"
  if [[ -n "${policy_path}" ]]; then
    echo "- Policy file: \`${policy_path}\`"
    echo "- Policy status: ${policy_status}"
  fi
  echo
  echo "## Requirement Status Matrix"
  echo
  echo "| Requirement | Evidence | Status | Note |"
  echo "| --- | --- | --- | --- |"
  for row in "${rows[@]}"; do
    IFS='|' read -r requirement evidence status note <<<"${row}"
    echo "| ${requirement} | ${evidence} | ${status} | ${note} |"
  done
  echo
  echo "## Remaining Work"
  echo
  echo "### Partial"
  if [[ "${#remaining_partial[@]}" -eq 0 ]]; then
    echo "- None."
  else
    for requirement in "${remaining_partial[@]}"; do
      echo "- ${requirement}"
      if [[ -n "${details["${requirement}"]}" ]]; then
        while IFS= read -r detail_line; do
          echo "  ${detail_line}"
        done <<<"${details["${requirement}"]}"
      fi
    done
  fi
  echo
  echo "### Planned"
  if [[ "${#remaining_planned[@]}" -eq 0 ]]; then
    echo "- None."
  else
    for requirement in "${remaining_planned[@]}"; do
      echo "- ${requirement}"
      if [[ -n "${details["${requirement}"]}" ]]; then
        while IFS= read -r detail_line; do
          echo "  ${detail_line}"
        done <<<"${details["${requirement}"]}"
      fi
    done
  fi
  echo
  echo "## Policy Evaluation"
  if [[ -z "${policy_path}" ]]; then
    echo "- Policy status: SKIPPED (no policy file provided)."
  else
    echo "- Policy status: ${policy_status}"
    echo "- Max partial: ${policy_max_partial}"
    echo "- Max planned: ${policy_max_planned}"
    echo "- Blocking prefixes: ${policy_blocking_prefixes:-none}"
    echo "- Blocking open count: ${policy_blocking_open_count}"
    echo "- Max blocking open: ${policy_max_blocking_open}"
    if [[ "${#policy_notes[@]}" -eq 0 ]]; then
      echo "- Policy notes: none."
    else
      for note in "${policy_notes[@]}"; do
        echo "- Policy note: ${note}"
      done
    fi
  fi
  echo
  echo "## Manifest Entries Outside Model"
  if [[ "${#unknown_manifest_requirements[@]}" -eq 0 ]]; then
    echo "- None."
  else
    for requirement in "${unknown_manifest_requirements[@]}"; do
      echo "- ${requirement}"
    done
  fi
  echo
} > "${output_path}"

echo "${overall_status}" > "${status_path}"
if [[ -n "${policy_status_path}" ]]; then
  echo "${policy_status}" > "${policy_status_path}"
fi
echo "Generated ${output_path} (status: ${overall_status})"

if [[ "${enforce_pass}" -eq 1 && "${overall_status}" != "PASS" ]]; then
  echo "Implementation trace open requirements:" >&2
  for row in "${rows[@]}"; do
    IFS='|' read -r requirement evidence status note <<<"${row}"
    if [[ "${status}" != "implemented" ]]; then
      echo "  - ${requirement}: ${status} (${evidence}; ${note})" >&2
      if [[ -n "${details["${requirement}"]}" ]]; then
        while IFS= read -r detail_line; do
          echo "      ${detail_line}" >&2
        done <<<"${details["${requirement}"]}"
      fi
    fi
  done
  echo "Implementation trace status is ${overall_status}; expected PASS." >&2
  exit 1
fi

if [[ "${enforce_policy}" -eq 1 && "${policy_status}" != "PASS" ]]; then
  echo "Implementation trace policy-open requirements:" >&2
  for row in "${rows[@]}"; do
    IFS='|' read -r requirement evidence status note <<<"${row}"
    if [[ "${status}" != "implemented" ]]; then
      echo "  - ${requirement}: ${status} (${evidence}; ${note})" >&2
      if [[ -n "${details["${requirement}"]}" ]]; then
        while IFS= read -r detail_line; do
          echo "      ${detail_line}" >&2
        done <<<"${details["${requirement}"]}"
      fi
    fi
  done
  if [[ "${#policy_notes[@]}" -gt 0 ]]; then
    echo "Policy notes:" >&2
    for note in "${policy_notes[@]}"; do
      echo "  - ${note}" >&2
    done
  fi
  echo "Implementation trace policy status is ${policy_status}; expected PASS." >&2
  exit 1
fi
