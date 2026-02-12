#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

echo "Smoke testing release/install scripts in dry-run mode..."
PF_DRY_RUN=1 "${REPO_ROOT}/scripts/build_vsix.sh"
PF_DRY_RUN=1 "${REPO_ROOT}/scripts/install_extension.sh"
bash "${REPO_ROOT}/scripts/verify_release_workflow_guardrails.sh"

echo "Script smoke tests passed."
