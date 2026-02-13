#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

echo "Smoke testing release/install scripts in dry-run mode..."
PF_DRY_RUN=1 "${REPO_ROOT}/scripts/build_vsix.sh"
PF_DRY_RUN=1 "${REPO_ROOT}/scripts/install_extension.sh"
bash "${REPO_ROOT}/scripts/verify_release_workflow_guardrails.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT
bash "${REPO_ROOT}/scripts/generate_dogfooding_reports.sh" "${tmp_dir}"
bash "${REPO_ROOT}/scripts/generate_dogfooding_triage_report.sh" "${tmp_dir}"
bash "${REPO_ROOT}/scripts/generate_obligation_reports.sh" "${tmp_dir}"
bash "${REPO_ROOT}/scripts/generate_adequacy_expectations.sh" \
  --selection "${REPO_ROOT}/models/system/adequacy_selection.env" \
  --output "${REPO_ROOT}/models/system/adequacy_expectations.tsv" \
  --check
bash "${REPO_ROOT}/scripts/run_adoption_demo.sh" "${tmp_dir}/adoption-demo" "${REPO_ROOT}/models/examples/sample.pf"
bash "${REPO_ROOT}/scripts/generate_pilot_evidence_report.sh" "${REPO_ROOT}/docs/adoption/pilot-evidence.tsv" "${tmp_dir}/adoption-pilot"
bash "${REPO_ROOT}/scripts/run_pf_quality_gate.sh" \
  --impact requirement:R009-A5-AgentAssistedModelExecution \
  --impact-hops 2 \
  "${REPO_ROOT}/models/system/tool_spec.pf"
bash "${REPO_ROOT}/scripts/run_lean_formal_check.sh" \
  --model "${REPO_ROOT}/models/system/tool_spec.pf" \
  --min-formalized-args 2 \
  --output-dir "${tmp_dir}/lean-formal"
bash "${REPO_ROOT}/scripts/run_lean_differential_check.sh" \
  --model "${REPO_ROOT}/models/system/tool_spec.pf" \
  --lean-status-json "${tmp_dir}/lean-formal/lean-check.json" \
  --output-dir "${tmp_dir}/lean-differential"
bash "${REPO_ROOT}/scripts/run_sysml_api_smoke.sh" "${tmp_dir}/sysml-api"
bash "${REPO_ROOT}/scripts/check_system_model.sh" "${tmp_dir}/system-model"
bash "${REPO_ROOT}/scripts/check_codex_self_model_contract.sh"
bash "${REPO_ROOT}/scripts/run_formal_backend_check.sh" "${tmp_dir}/formal-backend"

echo "Script smoke tests passed."
