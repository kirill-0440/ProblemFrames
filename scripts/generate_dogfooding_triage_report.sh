#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

INPUT_DIR="${DOGFOODING_INPUT_DIR:-${REPO_ROOT}/crates/pf_dsl/dogfooding}"
OUTPUT_DIR="${1:-${REPO_ROOT}/docs/dogfooding-triage}"
OUTPUT_FILE="${OUTPUT_DIR}/dogfooding-triage.md"
TRIAGE_MODE="${DOGFOODING_TRIAGE_MODE:-all}"
OWNERS_FILE="${DOGFOODING_OWNERS_FILE:-${REPO_ROOT}/.github/dogfooding-triage-owners.tsv}"

if [[ ! -d "${INPUT_DIR}" ]]; then
  echo "Dogfooding directory not found: ${INPUT_DIR}" >&2
  exit 1
fi

if [[ ! -f "${OWNERS_FILE}" ]]; then
  echo "Dogfooding owners mapping file not found: ${OWNERS_FILE}" >&2
  exit 1
fi

find_pf_models() {
  local mode="$1"

  if [[ "${mode}" == "core" ]]; then
    find "${INPUT_DIR}" \
      -type f \
      -name "*.pf" \
      ! -path "*/import_test/*" \
      ! -path "*/std_test/*" \
      | sort
  else
    find "${INPUT_DIR}" \
      -type f \
      -name "*.pf" \
      | sort
  fi
}

mkdir -p "${OUTPUT_DIR}"

raw_rows="$(mktemp)"
sorted_rows="$(mktemp)"
trap 'rm -f "${raw_rows}" "${sorted_rows}"' EXIT

trim_line() {
  local input="$1"
  sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//' <<<"${input}"
}

escape_md() {
  local input="$1"
  input="${input//|/\\|}"
  printf '%s' "${input}"
}

emit_row() {
  local model="$1"
  local req="$2"
  local frame="$3"
  local constrains="$4"
  local reference="$5"
  local constraint="$6"
  local priority="P3"
  local action="Review semantics coverage and add a targeted fixture."
  local owner="TBD"
  local due_date="TBD"
  local status="Open"

  if [[ "${frame}" == "CommandedBehavior" ]]; then
    priority="P1"
    action="Add/refresh command-origin fixture for \`${reference}\` -> \`${constrains}\`."
  elif [[ "${frame}" == "RequiredBehavior" ]]; then
    priority="P2"
    action="Add/refresh required-behavior fixture anchored on \`${constrains}\`."
  elif [[ "${frame}" == "InformationDisplay" ]]; then
    priority="P2"
    action="Add regression fixture for information projection rules."
  fi

  assign_owner_due_status "${model}" "${req}" "${priority}" owner due_date status

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${priority}" \
    "$(escape_md "${model}")" \
    "$(escape_md "${req}")" \
    "$(escape_md "${frame}")" \
    "$(escape_md "${action}")" \
    "$(escape_md "${constraint}")" \
    "$(escape_md "${owner}")" \
    "$(escape_md "${due_date}")" \
    "$(escape_md "${status}")" >> "${raw_rows}"
}

date_plus_days_utc() {
  local days="$1"
  if date -u -d "+${days} days" +"%Y-%m-%d" >/dev/null 2>&1; then
    date -u -d "+${days} days" +"%Y-%m-%d"
  else
    date -u -v+"${days}"d +"%Y-%m-%d"
  fi
}

default_due_days_for_priority() {
  local priority="$1"
  case "${priority}" in
    P1) echo 7 ;;
    P2) echo 14 ;;
    *) echo 21 ;;
  esac
}

assign_owner_due_status() {
  local model="$1"
  local req="$2"
  local priority="$3"
  local -n owner_ref="$4"
  local -n due_ref="$5"
  local -n status_ref="$6"
  local matched=0
  local line=""
  local model_re=""
  local req_re=""
  local owner_override=""
  local due_override=""
  local status_override=""

  owner_ref="TBD"
  due_ref="TBD"
  status_ref="Open"

  while IFS= read -r line || [[ -n "${line}" ]]; do
    line="$(trim_line "${line}")"
    if [[ -z "${line}" || "${line}" == \#* ]]; then
      continue
    fi

    IFS=$'\t' read -r model_re req_re owner_override due_override status_override <<< "${line}"
    if [[ -z "${model_re}" || -z "${req_re}" ]]; then
      continue
    fi

    if [[ "${model}" =~ ${model_re} && "${req}" =~ ${req_re} ]]; then
      matched=1
      if [[ -n "${owner_override}" ]]; then
        owner_ref="${owner_override}"
      fi
      if [[ -n "${status_override}" ]]; then
        status_ref="${status_override}"
      else
        status_ref="Planned"
      fi

      if [[ -n "${due_override}" ]]; then
        due_ref="$(date_plus_days_utc "${due_override}")"
      else
        due_ref="$(date_plus_days_utc "$(default_due_days_for_priority "${priority}")")"
      fi
      break
    fi
  done < "${OWNERS_FILE}"

  if [[ "${matched}" -eq 0 ]]; then
    due_ref="$(date_plus_days_utc "$(default_due_days_for_priority "${priority}")")"
    status_ref="Planned"
  fi
}

while IFS= read -r model_path; do
  model_rel="${model_path#${INPUT_DIR}/}"
  in_req=0
  req=""
  frame=""
  constrains=""
  reference=""
  constraint=""

  while IFS= read -r raw_line || [[ -n "${raw_line}" ]]; do
    line="${raw_line%%//*}"
    line="$(trim_line "${line}")"
    if [[ -z "${line}" ]]; then
      continue
    fi

    if [[ "${in_req}" -eq 0 ]]; then
      if [[ "${line}" =~ ^requirement[[:space:]]+\"([^\"]+)\" ]]; then
        in_req=1
        req="${BASH_REMATCH[1]}"
        frame="Unknown"
        constrains=""
        reference=""
        constraint=""
      fi
      continue
    fi

    if [[ "${line}" =~ ^frame:[[:space:]]*\"([^\"]+)\" ]]; then
      frame="${BASH_REMATCH[1]}"
      continue
    fi

    if [[ "${line}" =~ ^frame:[[:space:]]*([^[:space:]]+) ]]; then
      frame="${BASH_REMATCH[1]}"
      continue
    fi
    if [[ "${line}" =~ ^constrains:[[:space:]]*([^[:space:]]+) ]]; then
      constrains="${BASH_REMATCH[1]}"
      continue
    fi
    if [[ "${line}" =~ ^reference:[[:space:]]*([^[:space:]]+) ]]; then
      reference="${BASH_REMATCH[1]}"
      continue
    fi
    if [[ "${line}" =~ ^constraint:[[:space:]]*\"(.*)\"$ ]]; then
      constraint="${BASH_REMATCH[1]}"
      continue
    elif [[ "${line}" =~ ^constraint:[[:space:]]*'(.*)'$ ]]; then
      constraint="${BASH_REMATCH[1]}"
      continue
    elif [[ "${line}" =~ ^constraint:[[:space:]]*(.+)$ ]]; then
      constraint="${BASH_REMATCH[1]}"
      continue
    fi
    if [[ "${line}" =~ ^\} ]]; then
      emit_row "${model_rel}" "${req}" "${frame}" "${constrains}" "${reference}" "${constraint}"
      in_req=0
    fi
  done < "${model_path}"
done < <(
  find_pf_models "${TRIAGE_MODE}"
)

sort -t$'\t' -k1,1 -k2,2 -k3,3 "${raw_rows}" > "${sorted_rows}"

generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
total_actions="$(wc -l < "${sorted_rows}" | tr -d ' ')"
p1_count="$(awk -F'\t' '$1=="P1"{c++} END{print c+0}' "${sorted_rows}")"
p2_count="$(awk -F'\t' '$1=="P2"{c++} END{print c+0}' "${sorted_rows}")"
p3_count="$(awk -F'\t' '$1=="P3"{c++} END{print c+0}' "${sorted_rows}")"

{
  echo "# Dogfooding Triage Backlog"
  echo
  echo "- Generated (UTC): \`${generated_at}\`"
  echo "- Source: \`${INPUT_DIR}\`"
  echo
  echo "## Priority Summary"
  echo
  echo "| Bucket | Count |"
  echo "| --- | ---: |"
  echo "| P1 | ${p1_count} |"
  echo "| P2 | ${p2_count} |"
  echo "| P3 | ${p3_count} |"
  echo "| Total | ${total_actions} |"
  echo
  echo "## Top Actions"
  echo
  echo "| Priority | Model | Requirement | Frame | Proposed Action | Constraint Context | Owner | Due Date | Status |"
  echo "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"

  if [[ "${total_actions}" -eq 0 ]]; then
    echo "| n/a | n/a | n/a | n/a | No requirements found in dogfooding models. | n/a | n/a | n/a | n/a |"
  else
    row_count=0
    while IFS=$'\t' read -r priority model requirement frame_name action_text constraint_text owner due status; do
      row_count=$((row_count + 1))
      if [[ "${row_count}" -gt 12 ]]; then
        break
      fi
      printf '| %s | `%s` | `%s` | `%s` | %s | %s | %s | %s | %s |\n' \
        "${priority}" \
        "${model}" \
        "${requirement}" \
        "${frame_name}" \
        "${action_text}" \
        "${constraint_text}" \
        "${owner}" \
        "${due}" \
        "${status}"
    done < "${sorted_rows}"
  fi

  echo
  echo "## Usage"
  echo
  echo "- Review this table in weekly triage."
  echo "- Owners/due dates are auto-assigned from \`.github/dogfooding-triage-owners.tsv\`."
  echo "- Adjust routing/SLA by updating mapping rules in that file."
  echo "- Replan or close stale actions in the next cycle."
} > "${OUTPUT_FILE}"

echo "Generated ${OUTPUT_FILE}"
