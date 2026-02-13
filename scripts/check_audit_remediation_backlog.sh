#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/check_audit_remediation_backlog.sh [options]

Options:
  --model <path>        PF model path (default: models/system/tool_spec.pf)
  --backlog <path>      Audit remediation backlog TSV (default: models/system/audit_remediation_backlog.tsv)
  --output <path>       Markdown report output path (default: .ci-artifacts/audit-remediation/backlog.md)
  --json <path>         JSON report output path (default: .ci-artifacts/audit-remediation/backlog.json)
  --status-file <path>  Status file path (default: .ci-artifacts/audit-remediation/backlog.status)
  --enforce-pass        Exit non-zero when status is not PASS
  -h, --help            Show this help
USAGE
}

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
BACKLOG_FILE="${REPO_ROOT}/models/system/audit_remediation_backlog.tsv"
OUTPUT_FILE="${REPO_ROOT}/.ci-artifacts/audit-remediation/backlog.md"
JSON_FILE="${REPO_ROOT}/.ci-artifacts/audit-remediation/backlog.json"
STATUS_FILE="${REPO_ROOT}/.ci-artifacts/audit-remediation/backlog.status"
ENFORCE_PASS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      MODEL_FILE="$2"
      shift 2
      ;;
    --backlog)
      BACKLOG_FILE="$2"
      shift 2
      ;;
    --output)
      OUTPUT_FILE="$2"
      shift 2
      ;;
    --json)
      JSON_FILE="$2"
      shift 2
      ;;
    --status-file)
      STATUS_FILE="$2"
      shift 2
      ;;
    --enforce-pass)
      ENFORCE_PASS=1
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

if [[ "${MODEL_FILE}" != /* ]]; then
  MODEL_FILE="${REPO_ROOT}/${MODEL_FILE}"
fi
if [[ "${BACKLOG_FILE}" != /* ]]; then
  BACKLOG_FILE="${REPO_ROOT}/${BACKLOG_FILE}"
fi
if [[ "${OUTPUT_FILE}" != /* ]]; then
  OUTPUT_FILE="${REPO_ROOT}/${OUTPUT_FILE}"
fi
if [[ "${JSON_FILE}" != /* ]]; then
  JSON_FILE="${REPO_ROOT}/${JSON_FILE}"
fi
if [[ "${STATUS_FILE}" != /* ]]; then
  STATUS_FILE="${REPO_ROOT}/${STATUS_FILE}"
fi

mkdir -p "$(dirname -- "${OUTPUT_FILE}")"
mkdir -p "$(dirname -- "${JSON_FILE}")"
mkdir -p "$(dirname -- "${STATUS_FILE}")"

trim_field() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "${value}"
}

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  value="${value//$'\r'/\\r}"
  value="${value//$'\t'/\\t}"
  printf '%s' "${value}"
}

status="OPEN"
reason=""
total_rows=0
error_count=0
planned_count=0
in_progress_count=0
blocked_count=0
done_count=0

declare -A model_requirements=()
declare -A seen_item_ids=()
declare -a backlog_rows=()
declare -a error_rows=()

tmp_requirements_tsv="$(mktemp)"
trap 'rm -f "${tmp_requirements_tsv}"' EXIT

if [[ ! -f "${MODEL_FILE}" ]]; then
  reason="model file is missing"
elif [[ ! -f "${BACKLOG_FILE}" ]]; then
  reason="audit remediation backlog file is missing"
else
  cargo run -p pf_dsl -- "${MODEL_FILE}" --requirements-tsv > "${tmp_requirements_tsv}"
  while IFS='|' read -r requirement_id _frame _layer _extra; do
    requirement_id="$(trim_field "${requirement_id:-}")"
    if [[ -z "${requirement_id}" || "${requirement_id:0:1}" == "#" ]]; then
      continue
    fi
    model_requirements["${requirement_id}"]=1
  done < "${tmp_requirements_tsv}"

  while IFS='|' read -r item_id requirement_id priority status_value owner note extra; do
    item_id="$(trim_field "${item_id:-}")"
    requirement_id="$(trim_field "${requirement_id:-}")"
    priority="$(trim_field "${priority:-}")"
    status_value="$(trim_field "${status_value:-}")"
    owner="$(trim_field "${owner:-}")"
    note="$(trim_field "${note:-}")"
    extra="$(trim_field "${extra:-}")"

    if [[ -z "${item_id}" || "${item_id:0:1}" == "#" ]]; then
      continue
    fi
    total_rows=$((total_rows + 1))

    row_errors=()
    if [[ -n "${extra}" ]]; then
      row_errors+=("expected 6 TSV columns")
    fi
    if [[ -n "${seen_item_ids["${item_id}"]+x}" ]]; then
      row_errors+=("duplicate item id")
    fi
    seen_item_ids["${item_id}"]=1

    if [[ -z "${model_requirements["${requirement_id}"]+x}" ]]; then
      row_errors+=("unknown requirement id")
    fi
    case "${priority}" in
      P0|P1|P2|P3) ;;
      *) row_errors+=("invalid priority (expected P0..P3)") ;;
    esac
    case "${status_value}" in
      planned)
        planned_count=$((planned_count + 1))
        ;;
      in_progress)
        in_progress_count=$((in_progress_count + 1))
        ;;
      blocked)
        blocked_count=$((blocked_count + 1))
        ;;
      done)
        done_count=$((done_count + 1))
        ;;
      *)
        row_errors+=("invalid status (expected planned|in_progress|blocked|done)")
        ;;
    esac
    if [[ -z "${owner}" ]]; then
      row_errors+=("missing owner")
    fi
    if [[ -z "${note}" ]]; then
      row_errors+=("missing note")
    fi

    backlog_rows+=("${item_id}|${requirement_id}|${priority}|${status_value}|${owner}|${note}")
    if [[ "${#row_errors[@]}" -gt 0 ]]; then
      error_count=$((error_count + 1))
      IFS='; ' error_rows+=("${item_id}: ${row_errors[*]}")
      unset IFS
    fi
  done < "${BACKLOG_FILE}"

  if [[ "${total_rows}" -eq 0 ]]; then
    reason="audit remediation backlog is empty"
  elif [[ "${error_count}" -gt 0 ]]; then
    reason="${error_count} invalid remediation backlog row(s)"
  else
    status="PASS"
    reason="audit remediation backlog is valid and traceable"
  fi
fi

{
  echo "# Audit Remediation Backlog"
  echo
  echo "- Model: \`${MODEL_FILE}\`"
  echo "- Backlog file: \`${BACKLOG_FILE}\`"
  echo "- Status: \`${status}\`"
  echo "- Reason: ${reason}"
  echo "- Rows: ${total_rows}"
  echo "- Planned: ${planned_count}"
  echo "- In progress: ${in_progress_count}"
  echo "- Blocked: ${blocked_count}"
  echo "- Done: ${done_count}"
  echo
  echo "## Entries"
  echo
  echo "| Item | Requirement | Priority | Status | Owner | Note |"
  echo "| --- | --- | --- | --- | --- | --- |"
  if [[ "${#backlog_rows[@]}" -eq 0 ]]; then
    echo "| - | - | - | - | - | no entries |"
  else
    for row in "${backlog_rows[@]}"; do
      IFS='|' read -r item_id requirement_id priority status_value owner note <<< "${row}"
      echo "| \`${item_id}\` | \`${requirement_id}\` | ${priority} | ${status_value} | \`${owner}\` | ${note} |"
    done
  fi
  if [[ "${#error_rows[@]}" -gt 0 ]]; then
    echo
    echo "## Validation Errors"
    echo
    for row in "${error_rows[@]}"; do
      echo "- ${row}"
    done
  fi
} > "${OUTPUT_FILE}"

{
  echo "{"
  echo "  \"status\": \"${status}\","
  echo "  \"reason\": \"$(json_escape "${reason}")\","
  echo "  \"model\": \"$(json_escape "${MODEL_FILE}")\","
  echo "  \"backlog\": \"$(json_escape "${BACKLOG_FILE}")\","
  echo "  \"total_rows\": ${total_rows},"
  echo "  \"planned\": ${planned_count},"
  echo "  \"in_progress\": ${in_progress_count},"
  echo "  \"blocked\": ${blocked_count},"
  echo "  \"done\": ${done_count},"
  echo "  \"error_count\": ${error_count}"
  echo "}"
} > "${JSON_FILE}"

echo "${status}" > "${STATUS_FILE}"
echo "Generated ${OUTPUT_FILE}"
echo "Generated ${JSON_FILE}"

if [[ "${ENFORCE_PASS}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Audit remediation backlog status is ${status}; expected PASS." >&2
  exit 1
fi
