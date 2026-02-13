#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_adoption_demo.sh [output_dir] [model.pf]

Defaults:
  output_dir = .ci-artifacts/adoption-demo
  model.pf   = crates/pf_dsl/sample.pf
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

OUTPUT_DIR="${1:-${REPO_ROOT}/.ci-artifacts/adoption-demo}"
MODEL_FILE="${2:-${REPO_ROOT}/crates/pf_dsl/sample.pf}"

if [[ ! -f "${MODEL_FILE}" ]]; then
  echo "Model not found: ${MODEL_FILE}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

REPORT_FILE="${OUTPUT_DIR}/report.md"
OBLIGATIONS_FILE="${OUTPUT_DIR}/obligations.md"
CONCERN_FILE="${OUTPUT_DIR}/concern-coverage.md"
TRACEABILITY_FILE="${OUTPUT_DIR}/traceability.md"
DDD_FILE="${OUTPUT_DIR}/ddd-pim.md"
SYSML_FILE="${OUTPUT_DIR}/sysml2.txt"
WRSPM_FILE="${OUTPUT_DIR}/wrspm.md"
SUMMARY_FILE="${OUTPUT_DIR}/summary.md"

cargo run -p pf_dsl -- "${MODEL_FILE}" --report > "${REPORT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --obligations > "${OBLIGATIONS_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --concern-coverage > "${CONCERN_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-md > "${TRACEABILITY_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --ddd-pim > "${DDD_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --sysml2-text > "${SYSML_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --wrspm-report > "${WRSPM_FILE}"

{
  echo "# Adoption Demo Bundle"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Model: \`${MODEL_FILE}\`"
  echo
  echo "## Artifacts"
  echo
  echo "- \`report.md\`"
  echo "- \`obligations.md\`"
  echo "- \`concern-coverage.md\`"
  echo "- \`traceability.md\`"
  echo "- \`ddd-pim.md\`"
  echo "- \`sysml2.txt\`"
  echo "- \`wrspm.md\`"
} > "${SUMMARY_FILE}"

echo "Generated ${SUMMARY_FILE}"
