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

grep -q '^requirement "R011-H1-SolverBackedAdequacyEvidence"' "${REQUIREMENTS_FILE}" || {
  echo "R011-H1 requirement declaration is missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q '^requirement "R011-H2-DiffBasedModelFirstGate"' "${REQUIREMENTS_FILE}" || {
  echo "R011-H2 requirement declaration is missing in ${REQUIREMENTS_FILE}" >&2
  exit 1
}

grep -q '^requirement "R011-H3-CommandLevelAdequacyCoverage"' "${REQUIREMENTS_FILE}" || {
  echo "R011-H3 requirement declaration is missing in ${REQUIREMENTS_FILE}" >&2
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

grep -q 'requirements: .*"R011-H1-SolverBackedAdequacyEvidence"' "${SUBPROBLEMS_FILE}" || {
  echo "R011-H1 is not mapped in subproblem decomposition in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

grep -q 'requirements: .*"R011-H2-DiffBasedModelFirstGate"' "${SUBPROBLEMS_FILE}" || {
  echo "R011-H2 is not mapped in subproblem decomposition in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

grep -q 'requirements: .*"R011-H3-CommandLevelAdequacyCoverage"' "${SUBPROBLEMS_FILE}" || {
  echo "R011-H3 is not mapped in subproblem decomposition in ${SUBPROBLEMS_FILE}" >&2
  exit 1
}

ADEQUACY_EXPECTATIONS_FILE="${REPO_ROOT}/models/system/adequacy_expectations.tsv"
if [[ ! -f "${ADEQUACY_EXPECTATIONS_FILE}" ]]; then
  echo "Adequacy expectations manifest is missing: ${ADEQUACY_EXPECTATIONS_FILE}" >&2
  exit 1
fi
grep -q '^models/dogfooding/adequacy/pass.pf|Obl_A_exec|UNSAT|.*|required$' "${ADEQUACY_EXPECTATIONS_FILE}" || {
  echo "Adequacy expectations manifest is missing required pass obligation rule" >&2
  exit 1
}
grep -q '^models/dogfooding/adequacy/fail.pf|Obl_A_exec|UNSAT|.*|required$' "${ADEQUACY_EXPECTATIONS_FILE}" || {
  echo "Adequacy expectations manifest is missing required fail coverage rule" >&2
  exit 1
}

# 2) Diff-based model-first contract: implementation changes must include canonical model updates.
if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  base_ref="${PF_MODEL_FIRST_BASE_REF:-}"
  if [[ -z "${base_ref}" ]]; then
    for candidate in origin/main main origin/master master; do
      if git -C "${REPO_ROOT}" rev-parse --verify "${candidate}" >/dev/null 2>&1; then
        base_ref="${candidate}"
        break
      fi
    done
  fi

  base_commit=""
  if [[ -n "${base_ref}" ]]; then
    base_commit="$(git -C "${REPO_ROOT}" merge-base HEAD "${base_ref}" 2>/dev/null || true)"
  fi
  if [[ -z "${base_commit}" ]]; then
    base_commit="$(git -C "${REPO_ROOT}" rev-parse --verify HEAD~1 2>/dev/null || true)"
  fi

  if [[ -n "${base_commit}" ]]; then
    mapfile -t changed_files < <(
      git -C "${REPO_ROOT}" diff --name-only --diff-filter=ACMR "${base_commit}...HEAD" || true
    )

    canonical_model_changed=0
    implementation_changed=0
    implementation_files=()
    for changed in "${changed_files[@]}"; do
      if [[ "${changed}" =~ ^models/system/.*\.pf$ ]]; then
        canonical_model_changed=1
      fi
      if [[ "${changed}" =~ ^(crates/|scripts/|editors/|metamodel/|theory/|\.github/workflows/) ]]; then
        implementation_changed=1
        implementation_files+=("${changed}")
      fi
    done

    if [[ "${implementation_changed}" -eq 1 && "${canonical_model_changed}" -eq 0 ]]; then
      echo "Model-first contract violation: implementation files changed without canonical models/system/*.pf updates." >&2
      echo "Base reference: ${base_ref:-HEAD~1}" >&2
      for changed in "${implementation_files[@]}"; do
        echo " - ${changed}" >&2
      done
      exit 1
    fi
  fi
fi

# 3) Repository layout contract: all PF models and fixtures must live under models/.
pf_outside_models="$(rg --files -g '**/*.pf' | rg -v '^models/' || true)"
if [[ -n "${pf_outside_models}" ]]; then
  echo "Found .pf files outside models/:"
  echo "${pf_outside_models}"
  exit 1
fi

# 4) Requirement layer contract: each requirement has an explicit CIM/PIM/PSM layer.
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

# 5) Executable contract: impact path for R009-A5/R009-A6 must resolve through traceability mode.

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
