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
DDD_PIM_FILE="${OUTPUT_DIR}/tool_spec.ddd-pim.md"
SYSML2_TEXT_FILE="${OUTPUT_DIR}/tool_spec.sysml2.txt"
SYSML2_JSON_FILE="${OUTPUT_DIR}/tool_spec.sysml2.json"
TRACE_MAP_JSON_FILE="${OUTPUT_DIR}/tool_spec.trace-map.json"
DOT_FILE="${OUTPUT_DIR}/tool_spec.dot"
ALLOY_FILE="${OUTPUT_DIR}/tool_spec.als"
TRACEABILITY_MD_FILE="${OUTPUT_DIR}/tool_spec.traceability.md"
TRACEABILITY_CSV_FILE="${OUTPUT_DIR}/tool_spec.traceability.csv"
ADEQUACY_DIFFERENTIAL_FILE="${OUTPUT_DIR}/tool_spec.adequacy-differential.md"
ADEQUACY_JSON_FILE="${OUTPUT_DIR}/tool_spec.adequacy-evidence.json"
ADEQUACY_STATUS_FILE="${OUTPUT_DIR}/tool_spec.adequacy.status"
IMPLEMENTATION_TRACE_FILE="${OUTPUT_DIR}/tool_spec.implementation-trace.md"
IMPLEMENTATION_TRACE_STATUS_FILE="${OUTPUT_DIR}/tool_spec.implementation-trace.status"
IMPLEMENTATION_TRACE_POLICY_STATUS_FILE="${OUTPUT_DIR}/tool_spec.implementation-trace.policy.status"
WRSPM_REPORT_FILE="${OUTPUT_DIR}/tool_spec.wrspm.md"
WRSPM_JSON_FILE="${OUTPUT_DIR}/tool_spec.wrspm.json"
LEAN_MODEL_FILE="${OUTPUT_DIR}/tool_spec.lean"
LEAN_CHECK_JSON_FILE="${OUTPUT_DIR}/tool_spec.lean-check.json"
LEAN_CHECK_STATUS_FILE="${OUTPUT_DIR}/tool_spec.lean-check.status"
LEAN_DIFFERENTIAL_FILE="${OUTPUT_DIR}/tool_spec.lean-differential.md"
LEAN_DIFFERENTIAL_JSON_FILE="${OUTPUT_DIR}/tool_spec.lean-differential.json"
LEAN_DIFFERENTIAL_STATUS_FILE="${OUTPUT_DIR}/tool_spec.lean-differential.status"
IMPLEMENTATION_POLICY_FILE="${REPO_ROOT}/models/system/implementation_trace_policy.env"

cargo run -p pf_dsl -- "${MODEL_FILE}" --report > "${REPORT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --decomposition-closure > "${DECOMPOSITION_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --obligations > "${OBLIGATIONS_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --concern-coverage > "${CONCERN_COVERAGE_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --ddd-pim > "${DDD_PIM_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --sysml2-text > "${SYSML2_TEXT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --sysml2-json > "${SYSML2_JSON_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --trace-map-json > "${TRACE_MAP_JSON_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --dot > "${DOT_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --alloy > "${ALLOY_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-md > "${TRACEABILITY_MD_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --traceability-csv > "${TRACEABILITY_CSV_FILE}"
bash "${REPO_ROOT}/scripts/run_adequacy_evidence.sh" \
  --output "${ADEQUACY_DIFFERENTIAL_FILE}" \
  --json "${ADEQUACY_JSON_FILE}" \
  --status-file "${ADEQUACY_STATUS_FILE}"
bash "${REPO_ROOT}/scripts/check_model_implementation_trace.sh" \
  --traceability-csv "${TRACEABILITY_CSV_FILE}" \
  --output "${IMPLEMENTATION_TRACE_FILE}" \
  --status-file "${IMPLEMENTATION_TRACE_STATUS_FILE}" \
  --policy "${IMPLEMENTATION_POLICY_FILE}" \
  --policy-status-file "${IMPLEMENTATION_TRACE_POLICY_STATUS_FILE}" \
  --enforce-policy \
  "${MODEL_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --lean-model > "${LEAN_MODEL_FILE}"
bash "${REPO_ROOT}/scripts/run_lean_formal_check.sh" \
  --model "${MODEL_FILE}" \
  --output-dir "${OUTPUT_DIR}/lean-formal"
bash "${REPO_ROOT}/scripts/run_lean_differential_check.sh" \
  --model "${MODEL_FILE}" \
  --lean-status-json "${OUTPUT_DIR}/lean-formal/lean-check.json" \
  --output "${LEAN_DIFFERENTIAL_FILE}" \
  --json "${LEAN_DIFFERENTIAL_JSON_FILE}" \
  --status-file "${LEAN_DIFFERENTIAL_STATUS_FILE}" \
  --output-dir "${OUTPUT_DIR}"
cp "${OUTPUT_DIR}/lean-formal/lean-check.json" "${LEAN_CHECK_JSON_FILE}"
cp "${OUTPUT_DIR}/lean-formal/lean-check.status" "${LEAN_CHECK_STATUS_FILE}"
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
trace_map_coverage_status="$(
  grep -E '"status": "' "${TRACE_MAP_JSON_FILE}" \
    | head -n 1 \
    | sed -E 's/.*"status": "([^"]+)".*/\1/'
)"
trace_map_coverage_status="${trace_map_coverage_status:-UNKNOWN}"
adequacy_status="$(cat "${ADEQUACY_STATUS_FILE}" 2>/dev/null || true)"
adequacy_status="${adequacy_status:-UNKNOWN}"
implementation_trace_status="$(cat "${IMPLEMENTATION_TRACE_STATUS_FILE}" 2>/dev/null || true)"
implementation_trace_status="${implementation_trace_status:-UNKNOWN}"
implementation_trace_policy_status="$(cat "${IMPLEMENTATION_TRACE_POLICY_STATUS_FILE}" 2>/dev/null || true)"
implementation_trace_policy_status="${implementation_trace_policy_status:-UNKNOWN}"
lean_check_status="$(cat "${LEAN_CHECK_STATUS_FILE}" 2>/dev/null || true)"
lean_check_status="${lean_check_status:-UNKNOWN}"
lean_differential_status="$(cat "${LEAN_DIFFERENTIAL_STATUS_FILE}" 2>/dev/null || true)"
lean_differential_status="${lean_differential_status:-UNKNOWN}"

{
  echo "# System Model Quality Gate"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Model: \`models/system/tool_spec.pf\`"
  echo "- Decomposition closure status: \`${closure_status}\`"
  echo "- Concern coverage status: \`${concern_coverage_status}\`"
  echo "- Trace-map coverage status: \`${trace_map_coverage_status}\`"
  echo "- Adequacy evidence status: \`${adequacy_status}\`"
  echo "- Implementation trace status: \`${implementation_trace_status}\`"
  echo "- Implementation trace policy status: \`${implementation_trace_policy_status}\`"
  echo "- Lean formal check status: \`${lean_check_status}\`"
  echo "- Lean differential status: \`${lean_differential_status}\`"
  echo
  echo "## Artifacts"
  echo
  echo "- \`tool_spec.report.md\`"
  echo "- \`tool_spec.decomposition-closure.md\`"
  echo "- \`tool_spec.obligations.md\`"
  echo "- \`tool_spec.concern-coverage.md\`"
  echo "- \`tool_spec.ddd-pim.md\`"
  echo "- \`tool_spec.sysml2.txt\`"
  echo "- \`tool_spec.sysml2.json\`"
  echo "- \`tool_spec.trace-map.json\`"
  echo "- \`tool_spec.dot\`"
  echo "- \`tool_spec.als\`"
  echo "- \`tool_spec.traceability.md\`"
  echo "- \`tool_spec.traceability.csv\`"
  echo "- \`tool_spec.adequacy-differential.md\`"
  echo "- \`tool_spec.adequacy-evidence.json\`"
  echo "- \`tool_spec.implementation-trace.md\`"
  echo "- \`tool_spec.implementation-trace.policy.status\`"
  echo "- \`tool_spec.lean\`"
  echo "- \`tool_spec.lean-check.json\`"
  echo "- \`tool_spec.lean-differential.md\`"
  echo "- \`tool_spec.lean-differential.json\`"
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

if [[ "${trace_map_coverage_status}" != "PASS" ]]; then
  echo "System model trace-map coverage failed (${trace_map_coverage_status})" >&2
  exit 1
fi
