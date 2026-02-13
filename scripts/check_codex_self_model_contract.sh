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

grep -q '^requirement "R009-A7-ModelDirectoryPFContainment"' "${REQUIREMENTS_FILE}" || {
  echo "R009-A7 requirement declaration is missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q '@mda.layer("CIM")' "${REQUIREMENTS_FILE}" || {
  echo "CIM requirement layer marks are missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q '@mda.layer("PIM")' "${REQUIREMENTS_FILE}" || {
  echo "PIM requirement layer marks are missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q '@mda.layer("PSM")' "${REQUIREMENTS_FILE}" || {
  echo "PSM requirement layer marks are missing in ${REQUIREMENTS_FILE}" >&2
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

grep -q 'requirements: .*"R009-A7-ModelDirectoryPFContainment"' "${SUBPROBLEMS_FILE}" || {
  echo "R009-A7 is not mapped in subproblem decomposition in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

# 2) Repository layout contract: all PF models and fixtures must live under models/.
pf_outside_models="$(rg --files -g '**/*.pf' | rg -v '^models/' || true)"
if [[ -n "${pf_outside_models}" ]]; then
  echo "Found .pf files outside models/:"
  echo "${pf_outside_models}"
  exit 1
fi

# 3) Requirement layer contract: each requirement has an explicit CIM/PIM/PSM layer.
requirements_tsv_file="$(mktemp)"
traceability_file="$(mktemp)"
trap 'rm -f "${requirements_tsv_file}" "${traceability_file}"' EXIT

cargo run -p pf_dsl -- "${MODEL_FILE}" --requirements-tsv > "${requirements_tsv_file}"

invalid_requirements_layer_rows="$(awk -F'|' '
  $0 ~ /^#/ || NF == 0 { next }
  NF != 3 { print $0; next }
  $3 != "CIM" && $3 != "PIM" && $3 != "PSM" { print $0 }
' "${requirements_tsv_file}")"
if [[ -n "${invalid_requirements_layer_rows}" ]]; then
  echo "Requirements TSV contains invalid layer rows:"
  echo "${invalid_requirements_layer_rows}"
  exit 1
fi

# 4) Executable contract: impact path for R009-A5/R009-A6 must resolve through traceability mode.

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
