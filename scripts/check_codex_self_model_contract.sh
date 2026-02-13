#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
DOMAINS_FILE="${REPO_ROOT}/models/system/domains.pf"
REQUIREMENTS_FILE="${REPO_ROOT}/models/system/requirements.pf"
SUBPROBLEMS_FILE="${REPO_ROOT}/models/system/subproblems.pf"

if [[ ! -f "${MODEL_FILE}" ]]; then
  echo "System model not found: ${MODEL_FILE}" >&2
  exit 1
fi

# 1) Structural anchors in the canonical model modules.
grep -q '^domain Codex kind biddable role given' "${DOMAINS_FILE}" || {
  echo "Codex domain declaration is missing or malformed in ${DOMAINS_FILE}" >&2
  exit 1
}

grep -q '^requirement "R009-A5-AgentAssistedModelExecution"' "${REQUIREMENTS_FILE}" || {
  echo "R009-A5 requirement declaration is missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q '^requirement "R009-A6-ModelFirstChangeControl"' "${REQUIREMENTS_FILE}" || {
  echo "R009-A6 requirement declaration is missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q 'participants: .*\bCodex\b' "${SUBPROBLEMS_FILE}" || {
  echo "No subproblem participants include Codex in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

grep -q 'requirements: .*"R009-A5-AgentAssistedModelExecution"' "${SUBPROBLEMS_FILE}" || {
  echo "R009-A5 is not mapped in subproblem decomposition in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

grep -q 'requirements: .*"R009-A6-ModelFirstChangeControl"' "${SUBPROBLEMS_FILE}" || {
  echo "R009-A6 is not mapped in subproblem decomposition in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

# 2) Executable contract: impact path for R009-A5 must resolve through traceability mode.
traceability_file="$(mktemp)"
trap 'rm -f "${traceability_file}"' EXIT

cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-md \
  --impact=requirement:R009-A5-AgentAssistedModelExecution \
  --impact-hops=2 > "${traceability_file}"

impact_line="$(grep -F '`requirement:R009-A5-AgentAssistedModelExecution` -> ' "${traceability_file}" || true)"
if [[ -z "${impact_line}" || "${impact_line}" != *"R009-A5-AgentAssistedModelExecution"* ]]; then
  echo "Impact trace for R009-A5 is missing in traceability export" >&2
  exit 1
fi

cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-md \
  --impact=requirement:R009-A6-ModelFirstChangeControl \
  --impact-hops=2 > "${traceability_file}"

impact_line="$(grep -F '`requirement:R009-A6-ModelFirstChangeControl` -> ' "${traceability_file}" || true)"
if [[ -z "${impact_line}" || "${impact_line}" != *"R009-A6-ModelFirstChangeControl"* ]]; then
  echo "Impact trace for R009-A6 is missing in traceability export" >&2
  exit 1
fi

echo "Codex self-model contract checks passed."
