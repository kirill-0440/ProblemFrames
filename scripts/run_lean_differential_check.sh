#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_lean_differential_check.sh [options]

Options:
  --model <path>             PF model path (default: models/system/tool_spec.pf)
  --lean-status-json <path>  Existing Lean status JSON (default: run lean formal check first)
  --output-dir <dir>         Output directory (default: .ci-artifacts/lean-differential)
  --output <path>            Markdown output path
  --json <path>              JSON output path
  --status-file <path>       Status file output path
  --enforce-pass             Exit non-zero when differential status is not PASS
  -h, --help                 Show this help
USAGE
}

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
LEAN_STATUS_JSON=""
OUTPUT_DIR="${REPO_ROOT}/.ci-artifacts/lean-differential"
OUTPUT_FILE=""
JSON_FILE=""
STATUS_FILE=""
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      MODEL_FILE="$2"
      shift 2
      ;;
    --lean-status-json)
      LEAN_STATUS_JSON="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --output)
      OUTPUT_FILE="$2"
      shift 2
      ;;
    --json)
      JSON_FILE="$2"
      shift 2
      ;;
    --status-file)
      STATUS_FILE="$2"
      shift 2
      ;;
    --enforce-pass)
      enforce_pass=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ "${MODEL_FILE}" != /* ]]; then
  MODEL_FILE="${REPO_ROOT}/${MODEL_FILE}"
fi
if [[ "${OUTPUT_DIR}" != /* ]]; then
  OUTPUT_DIR="${REPO_ROOT}/${OUTPUT_DIR}"
fi
if [[ -n "${LEAN_STATUS_JSON}" && "${LEAN_STATUS_JSON}" != /* ]]; then
  LEAN_STATUS_JSON="${REPO_ROOT}/${LEAN_STATUS_JSON}"
fi
if [[ -n "${OUTPUT_FILE}" && "${OUTPUT_FILE}" != /* ]]; then
  OUTPUT_FILE="${REPO_ROOT}/${OUTPUT_FILE}"
fi
if [[ -n "${JSON_FILE}" && "${JSON_FILE}" != /* ]]; then
  JSON_FILE="${REPO_ROOT}/${JSON_FILE}"
fi
if [[ -n "${STATUS_FILE}" && "${STATUS_FILE}" != /* ]]; then
  STATUS_FILE="${REPO_ROOT}/${STATUS_FILE}"
fi

if [[ ! -f "${MODEL_FILE}" ]]; then
  echo "Model not found: ${MODEL_FILE}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

OUTPUT_FILE="${OUTPUT_FILE:-${OUTPUT_DIR}/lean-differential.md}"
JSON_FILE="${JSON_FILE:-${OUTPUT_DIR}/lean-differential.json}"
STATUS_FILE="${STATUS_FILE:-${OUTPUT_DIR}/lean-differential.status}"

if [[ -z "${LEAN_STATUS_JSON}" ]]; then
  lean_check_dir="${OUTPUT_DIR}/lean-check"
  bash "${REPO_ROOT}/scripts/run_lean_formal_check.sh" \
    --model "${MODEL_FILE}" \
    --output-dir "${lean_check_dir}"
  LEAN_STATUS_JSON="${lean_check_dir}/lean-check.json"
fi

if [[ ! -f "${LEAN_STATUS_JSON}" ]]; then
  echo "Lean status JSON not found: ${LEAN_STATUS_JSON}" >&2
  exit 1
fi

rust_verdict="ERROR"
if concern_output="$(cargo run -p pf_dsl -- "${MODEL_FILE}" --concern-coverage 2>&1)"; then
  rust_verdict="$(
    printf '%s\n' "${concern_output}" \
      | grep -E '^- Concern coverage status: ' \
      | sed -e 's/^- Concern coverage status: //'
  )"
  rust_verdict="${rust_verdict:-ERROR}"
fi

lean_verdict="$(grep -E '"lean_verdict": "' "${LEAN_STATUS_JSON}" | sed -E 's/.*"lean_verdict": "([^"]+)".*/\1/')"
lean_verdict="${lean_verdict:-UNKNOWN}"
coverage_status="$(grep -E '"coverage_status": "' "${LEAN_STATUS_JSON}" | sed -E 's/.*"coverage_status": "([^"]+)".*/\1/')"
coverage_status="${coverage_status:-UNKNOWN}"
formalized_count="$(grep -E '"formalized_count": ' "${LEAN_STATUS_JSON}" | sed -E 's/.*"formalized_count": *([0-9]+).*/\1/')"
formalized_count="${formalized_count:-0}"
min_formalized_args="$(grep -E '"min_formalized_args": ' "${LEAN_STATUS_JSON}" | sed -E 's/.*"min_formalized_args": *([0-9]+).*/\1/')"
min_formalized_args="${min_formalized_args:-0}"

category="mixed_non_pass"
status="OPEN"
match="false"

if [[ "${coverage_status}" != "PASS" ]]; then
  category="coverage_open"
  status="OPEN"
elif [[ "${rust_verdict}" == "${lean_verdict}" ]]; then
  case "${rust_verdict}" in
    PASS)
      category="both_pass"
      status="PASS"
      match="true"
      ;;
    FAIL)
      category="both_fail"
      status="PASS"
      match="true"
      ;;
    *)
      category="both_error"
      ;;
  esac
else
  case "${lean_verdict}" in
    SKIPPED)
      category="lean_skipped"
      ;;
    ERROR|UNKNOWN)
      category="lean_error"
      ;;
    PASS)
      category="lean_only_pass"
      ;;
    FAIL)
      if [[ "${rust_verdict}" == "PASS" ]]; then
        category="rust_only_pass"
      fi
      ;;
  esac
fi

{
  echo "# Lean Differential Report"
  echo
  echo "- Model: \`${MODEL_FILE#${REPO_ROOT}/}\`"
  echo "- Lean status source: \`${LEAN_STATUS_JSON#${REPO_ROOT}/}\`"
  echo "- Differential status: \`${status}\`"
  echo "- Coverage status: \`${coverage_status}\` (${formalized_count}/${min_formalized_args} required)"
  echo
  echo "| Rust Verdict | Lean Verdict | Coverage | Category | Match |"
  echo "| --- | --- | --- | --- | --- |"
  echo "| ${rust_verdict} | ${lean_verdict} | ${coverage_status} | ${category} | ${match} |"
} > "${OUTPUT_FILE}"

{
  echo "{"
  echo "  \"status\": \"${status}\","
  echo "  \"model\": \"${MODEL_FILE#${REPO_ROOT}/}\","
  echo "  \"lean_status_source\": \"${LEAN_STATUS_JSON#${REPO_ROOT}/}\","
  echo "  \"rust_verdict\": \"${rust_verdict}\","
  echo "  \"lean_verdict\": \"${lean_verdict}\","
  echo "  \"coverage_status\": \"${coverage_status}\","
  echo "  \"formalized_count\": ${formalized_count},"
  echo "  \"min_formalized_args\": ${min_formalized_args},"
  echo "  \"category\": \"${category}\","
  echo "  \"match\": ${match}"
  echo "}"
} > "${JSON_FILE}"

echo "${status}" > "${STATUS_FILE}"

echo "Generated ${OUTPUT_FILE}"
echo "Generated ${JSON_FILE}"

if [[ "${enforce_pass}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Lean differential status is ${status}; expected PASS." >&2
  exit 1
fi
