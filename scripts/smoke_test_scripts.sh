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
bash "${REPO_ROOT}/scripts/run_pf_quality_gate.sh" \
  --impact requirement:R009-A5-AgentAssistedModelExecution \
  --impact-hops 2 \
  "${REPO_ROOT}/models/system/tool_spec.pf"
bash "${REPO_ROOT}/scripts/check_system_model.sh" "${tmp_dir}/system-model"
bash "${REPO_ROOT}/scripts/check_codex_self_model_contract.sh"
bash "${REPO_ROOT}/scripts/run_formal_backend_check.sh" "${tmp_dir}/formal-backend"

echo "Script smoke tests passed."
