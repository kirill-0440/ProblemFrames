#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
OUTPUT_DIR="${1:-${REPO_ROOT}/.ci-artifacts/system-model}"
SUMMARY_FILE="${OUTPUT_DIR}/summary.md"

if [[ ! -f "${MODEL_FILE}" ]]; then
  echo "System model not found: ${MODEL_FILE}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

REPORT_FILE="${OUTPUT_DIR}/tool_spec.report.md"
OBLIGATIONS_FILE="${OUTPUT_DIR}/tool_spec.obligations.md"
DOT_FILE="${OUTPUT_DIR}/tool_spec.dot"
ALLOY_FILE="${OUTPUT_DIR}/tool_spec.als"

cargo run -p pf_dsl -- "${MODEL_FILE}" --report > "${REPORT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --obligations > "${OBLIGATIONS_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --dot > "${DOT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --alloy > "${ALLOY_FILE}"

{
  echo "# System Model Quality Gate"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Model: \`models/system/tool_spec.pf\`"
  echo
  echo "## Artifacts"
  echo
  echo "- \`tool_spec.report.md\`"
  echo "- \`tool_spec.obligations.md\`"
  echo "- \`tool_spec.dot\`"
  echo "- \`tool_spec.als\`"
} > "${SUMMARY_FILE}"

echo "Generated ${SUMMARY_FILE}"
