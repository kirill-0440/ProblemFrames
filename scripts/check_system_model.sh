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
DECOMPOSITION_FILE="${OUTPUT_DIR}/tool_spec.decomposition-closure.md"
OBLIGATIONS_FILE="${OUTPUT_DIR}/tool_spec.obligations.md"
CONCERN_COVERAGE_FILE="${OUTPUT_DIR}/tool_spec.concern-coverage.md"
DOT_FILE="${OUTPUT_DIR}/tool_spec.dot"
ALLOY_FILE="${OUTPUT_DIR}/tool_spec.als"
TRACEABILITY_MD_FILE="${OUTPUT_DIR}/tool_spec.traceability.md"
TRACEABILITY_CSV_FILE="${OUTPUT_DIR}/tool_spec.traceability.csv"
WRSPM_REPORT_FILE="${OUTPUT_DIR}/tool_spec.wrspm.md"
WRSPM_JSON_FILE="${OUTPUT_DIR}/tool_spec.wrspm.json"

cargo run -p pf_dsl -- "${MODEL_FILE}" --report > "${REPORT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --decomposition-closure > "${DECOMPOSITION_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --obligations > "${OBLIGATIONS_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --concern-coverage > "${CONCERN_COVERAGE_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --dot > "${DOT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --alloy > "${ALLOY_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-md > "${TRACEABILITY_MD_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-csv > "${TRACEABILITY_CSV_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --wrspm-report > "${WRSPM_REPORT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --wrspm-json > "${WRSPM_JSON_FILE}"
bash "${REPO_ROOT}/scripts/check_codex_self_model_contract.sh"

closure_status="$(
  grep -E "^- Closure status: " "${DECOMPOSITION_FILE}" \
    | sed -e 's/^- Closure status: //'
)"
closure_status="${closure_status:-UNKNOWN}"
concern_coverage_status="$(
  grep -E "^- Concern coverage status: " "${CONCERN_COVERAGE_FILE}" \
    | sed -e 's/^- Concern coverage status: //'
)"
concern_coverage_status="${concern_coverage_status:-UNKNOWN}"

{
  echo "# System Model Quality Gate"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Model: \`models/system/tool_spec.pf\`"
  echo "- Decomposition closure status: \`${closure_status}\`"
  echo "- Concern coverage status: \`${concern_coverage_status}\`"
  echo
  echo "## Artifacts"
  echo
  echo "- \`tool_spec.report.md\`"
  echo "- \`tool_spec.decomposition-closure.md\`"
  echo "- \`tool_spec.obligations.md\`"
  echo "- \`tool_spec.concern-coverage.md\`"
  echo "- \`tool_spec.dot\`"
  echo "- \`tool_spec.als\`"
  echo "- \`tool_spec.traceability.md\`"
  echo "- \`tool_spec.traceability.csv\`"
  echo "- \`tool_spec.wrspm.md\`"
  echo "- \`tool_spec.wrspm.json\`"
} > "${SUMMARY_FILE}"

echo "Generated ${SUMMARY_FILE}"

if [[ "${closure_status}" != "PASS" ]]; then
  echo "System model decomposition closure failed (${closure_status})" >&2
  exit 1
fi

if [[ "${concern_coverage_status}" != "PASS" ]]; then
  echo "System model concern coverage failed (${concern_coverage_status})" >&2
  exit 1
fi
