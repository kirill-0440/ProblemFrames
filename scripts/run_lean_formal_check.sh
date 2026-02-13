#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_lean_formal_check.sh [options]

Options:
  --model <path>        PF model path (default: models/system/tool_spec.pf)
  --output-dir <dir>    Output directory (default: .ci-artifacts/lean-formal)
  --min-formalized-args <n>
                        Minimum number of formalized correctness arguments required (default: 0)
  --gated               Run only if PF_LEAN_FORMAL_ENABLED=1
  --enforce-pass        Exit non-zero when status is not PASS
  -h, --help            Show this help

Environment:
  PF_LEAN_FORMAL_ENABLED  Gate variable used with --gated.
USAGE
}

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
OUTPUT_DIR="${REPO_ROOT}/.ci-artifacts/lean-formal"
min_formalized_args=0
gated=0
enforce_pass=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      MODEL_FILE="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --min-formalized-args)
      min_formalized_args="$2"
      shift 2
      ;;
    --gated)
      gated=1
      shift
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

if ! [[ "${min_formalized_args}" =~ ^[0-9]+$ ]]; then
  echo "Invalid value for --min-formalized-args: ${min_formalized_args}" >&2
  exit 1
fi

if [[ "${MODEL_FILE}" != /* ]]; then
  MODEL_FILE="${REPO_ROOT}/${MODEL_FILE}"
fi
if [[ "${OUTPUT_DIR}" != /* ]]; then
  OUTPUT_DIR="${REPO_ROOT}/${OUTPUT_DIR}"
fi

if [[ ! -f "${MODEL_FILE}" ]]; then
  echo "Model not found: ${MODEL_FILE}" >&2
  exit 1
fi

LEAN_MODEL_FILE="${OUTPUT_DIR}/model.lean"
LEAN_COVERAGE_FILE="${OUTPUT_DIR}/lean-coverage.json"
STATUS_JSON_FILE="${OUTPUT_DIR}/lean-check.json"
STATUS_FILE="${OUTPUT_DIR}/lean-check.status"
LOG_FILE="${OUTPUT_DIR}/lean-check.log"

mkdir -p "${OUTPUT_DIR}"

if [[ "${gated}" -eq 1 && "${PF_LEAN_FORMAL_ENABLED:-0}" != "1" ]]; then
  cat > "${STATUS_JSON_FILE}" <<JSON
{
  "status": "SKIPPED",
  "rust_verdict": "SKIPPED",
  "lean_verdict": "SKIPPED",
  "coverage_status": "SKIPPED",
  "formalized_count": 0,
  "skipped_count": 0,
  "total_correctness_arguments": 0,
  "min_formalized_args": ${min_formalized_args},
  "reason": "PF_LEAN_FORMAL_ENABLED is not set to 1"
}
JSON
  cat > "${LEAN_COVERAGE_FILE}" <<JSON
{
  "problem": "${MODEL_FILE#${REPO_ROOT}/}",
  "total_correctness_arguments": 0,
  "formalized_count": 0,
  "skipped_count": 0,
  "formalized": [],
  "skipped": []
}
JSON
  echo "SKIPPED" > "${STATUS_FILE}"
  {
    echo "Lean formal check skipped by gate."
    echo "Set PF_LEAN_FORMAL_ENABLED=1 to run Lean checks."
  } > "${LOG_FILE}"
  echo "Generated ${STATUS_JSON_FILE}"
  echo "Generated ${LEAN_COVERAGE_FILE}"
  echo "Generated ${LOG_FILE}"
  exit 0
fi

{
  echo "Running Lean formal check"
  echo "Model: ${MODEL_FILE}"
  echo "Output dir: ${OUTPUT_DIR}"
  echo "Generate command: cargo run -p pf_dsl -- ${MODEL_FILE} --lean-model"
} > "${LOG_FILE}"

cargo run -p pf_dsl -- "${MODEL_FILE}" --lean-model > "${LEAN_MODEL_FILE}" 2>> "${LOG_FILE}"
cargo run -p pf_dsl -- "${MODEL_FILE}" --lean-coverage-json > "${LEAN_COVERAGE_FILE}" 2>> "${LOG_FILE}"

extract_json_number() {
  local key="$1"
  local file="$2"
  local value
  value="$(grep -E "\"${key}\":" "${file}" | head -n 1 | sed -E 's/.*: *([0-9]+).*/\1/' || true)"
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    value="0"
  fi
  printf '%s' "${value}"
}

rust_verdict="ERROR"
if concern_output="$(cargo run -p pf_dsl -- "${MODEL_FILE}" --concern-coverage 2>> "${LOG_FILE}")"; then
  rust_verdict="$(
    printf '%s\n' "${concern_output}" \
      | grep -E '^- Concern coverage status: ' \
      | sed -e 's/^- Concern coverage status: //'
  )"
  rust_verdict="${rust_verdict:-ERROR}"
fi

formalized_count="$(extract_json_number "formalized_count" "${LEAN_COVERAGE_FILE}")"
skipped_count="$(extract_json_number "skipped_count" "${LEAN_COVERAGE_FILE}")"
total_correctness_arguments="$(extract_json_number "total_correctness_arguments" "${LEAN_COVERAGE_FILE}")"

coverage_status="PASS"
if [[ "${formalized_count}" -lt "${min_formalized_args}" ]]; then
  coverage_status="OPEN"
fi

lean_verdict="SKIPPED"
reason="lake is not available or theory project is missing"

if command -v lake >/dev/null 2>&1 && [[ -f "${REPO_ROOT}/theory/lakefile.lean" ]]; then
  if (cd -- "${REPO_ROOT}/theory" && lake env lean "${LEAN_MODEL_FILE}" >> "${LOG_FILE}" 2>&1); then
    lean_verdict="PASS"
    reason="lean model type-check passed"
  else
    lean_verdict="FAIL"
    reason="lean model type-check failed"
  fi
fi

status="OPEN"
if [[ "${lean_verdict}" == "PASS" && "${coverage_status}" == "PASS" ]]; then
  status="PASS"
fi

if [[ "${lean_verdict}" == "PASS" && "${coverage_status}" != "PASS" ]]; then
  reason="formalized coverage ${formalized_count} is below minimum ${min_formalized_args}"
fi

{
  echo "{"
  echo "  \"status\": \"${status}\","
  echo "  \"rust_verdict\": \"${rust_verdict}\","
  echo "  \"lean_verdict\": \"${lean_verdict}\","
  echo "  \"coverage_status\": \"${coverage_status}\","
  echo "  \"formalized_count\": ${formalized_count},"
  echo "  \"skipped_count\": ${skipped_count},"
  echo "  \"total_correctness_arguments\": ${total_correctness_arguments},"
  echo "  \"min_formalized_args\": ${min_formalized_args},"
  echo "  \"model\": \"${MODEL_FILE#${REPO_ROOT}/}\","
  echo "  \"lean_model\": \"${LEAN_MODEL_FILE#${REPO_ROOT}/}\","
  echo "  \"lean_coverage\": \"${LEAN_COVERAGE_FILE#${REPO_ROOT}/}\","
  echo "  \"reason\": \"${reason}\""
  echo "}"
} > "${STATUS_JSON_FILE}"

echo "${status}" > "${STATUS_FILE}"

echo "Generated ${LEAN_MODEL_FILE}"
echo "Generated ${LEAN_COVERAGE_FILE}"
echo "Generated ${STATUS_JSON_FILE}"
echo "Generated ${LOG_FILE}"

if [[ "${enforce_pass}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Lean formal check status is ${status}; expected PASS." >&2
  exit 1
fi
