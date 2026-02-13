#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

INPUT_DIR="${REPO_ROOT}/crates/pf_dsl/dogfooding"
OUTPUT_DIR="${REPO_ROOT}/docs/formal-backend"
SUMMARY_FILE=""
ENFORCE_SOLVER_PASS=0
SOLVER_NAME="sat4j"
REPEAT_COUNT=1
COMMAND_SELECTOR=""
ALLOY_JAR_PATH=""

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_formal_backend_check.sh [output-dir] [options]

Options:
  --enforce-solver-pass     Exit non-zero when any solver check is not PASS
  --solver <name>           Alloy solver id (default: sat4j)
  --repeat <n>              Number of solutions per command (default: 1)
  --command <pattern>       Optional command selector passed to Alloy CLI
  --jar-path <path>         Explicit Alloy CLI jar path
  -h, --help                Show this help

Environment:
  PF_FORMAL_TRACK_BLOCKING=1 enables --enforce-solver-pass by default.
USAGE
}

if [[ "${PF_FORMAL_TRACK_BLOCKING:-0}" == "1" ]]; then
  ENFORCE_SOLVER_PASS=1
fi

if [[ $# -gt 0 && "${1:-}" != -* ]]; then
  OUTPUT_DIR="$1"
  shift
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --enforce-solver-pass)
      ENFORCE_SOLVER_PASS=1
      shift
      ;;
    --solver)
      SOLVER_NAME="$2"
      shift 2
      ;;
    --repeat)
      REPEAT_COUNT="$2"
      shift 2
      ;;
    --command)
      COMMAND_SELECTOR="$2"
      shift 2
      ;;
    --jar-path)
      ALLOY_JAR_PATH="$2"
      shift 2
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

if ! [[ "${REPEAT_COUNT}" =~ ^[0-9]+$ ]] || [[ "${REPEAT_COUNT}" -lt 1 ]]; then
  echo "Invalid --repeat value: ${REPEAT_COUNT}" >&2
  exit 1
fi

if [[ ! -d "${INPUT_DIR}" ]]; then
  echo "Dogfooding directory not found: ${INPUT_DIR}" >&2
  exit 1
fi

if [[ "${OUTPUT_DIR}" != /* ]]; then
  OUTPUT_DIR="${REPO_ROOT}/${OUTPUT_DIR}"
fi
if [[ -n "${ALLOY_JAR_PATH}" && "${ALLOY_JAR_PATH}" != /* ]]; then
  ALLOY_JAR_PATH="${REPO_ROOT}/${ALLOY_JAR_PATH}"
fi

mkdir -p "${OUTPUT_DIR}"
SUMMARY_FILE="${OUTPUT_DIR}/formal-backend-summary.md"

generated_count=0
failed_count=0
solver_pass_count=0
solver_open_count=0
solver_unknown_count=0
solver_invocation_fail_count=0
solver_blocking_fail_count=0

{
  echo "# Formal Backend Check (Alloy)"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Source models: \`${INPUT_DIR}\`"
  echo "- Solver: \`${SOLVER_NAME}\`"
  echo "- Repeat: \`${REPEAT_COUNT}\`"
  if [[ -n "${COMMAND_SELECTOR}" ]]; then
    echo "- Command selector: \`${COMMAND_SELECTOR}\`"
  else
    echo "- Command selector: \`<all>\`"
  fi
  if [[ "${ENFORCE_SOLVER_PASS}" -eq 1 ]]; then
    echo "- Solver policy: \`blocking\`"
  else
    echo "- Solver policy: \`non-blocking\`"
  fi
  echo
  echo "## Artifacts"
  echo
} > "${SUMMARY_FILE}"

while IFS= read -r model; do
  rel_path="${model#${INPUT_DIR}/}"
  out_file="${OUTPUT_DIR}/${rel_path%.pf}.als"
  mkdir -p "$(dirname -- "${out_file}")"

  if cargo run -p pf_dsl -- "${model}" --alloy > "${out_file}"; then
    generated_count=$((generated_count + 1))
    echo "Generated ${out_file}"
  else
    failed_count=$((failed_count + 1))
    printf -- '- `%s` (generation failed)\n' "${rel_path%.pf}.als" >> "${SUMMARY_FILE}"
    echo "Failed to generate ${out_file}" >&2
    continue
  fi

  solver_dir="${OUTPUT_DIR}/solver/${rel_path%.pf}"
  solver_report="${solver_dir}/alloy-solver.md"
  solver_json="${solver_dir}/alloy-solver.json"
  solver_status_file="${solver_dir}/alloy-solver.status"

  solver_cmd=(
    bash "${REPO_ROOT}/scripts/run_alloy_solver_check.sh"
    --model "${model}"
    --alloy-file "${out_file}"
    --output-dir "${solver_dir}"
    --report "${solver_report}"
    --json "${solver_json}"
    --status-file "${solver_status_file}"
    --solver "${SOLVER_NAME}"
    --repeat "${REPEAT_COUNT}"
  )
  if [[ -n "${COMMAND_SELECTOR}" ]]; then
    solver_cmd+=(--command "${COMMAND_SELECTOR}")
  fi
  if [[ -n "${ALLOY_JAR_PATH}" ]]; then
    solver_cmd+=(--jar-path "${ALLOY_JAR_PATH}")
  fi
  if [[ "${ENFORCE_SOLVER_PASS}" -eq 1 ]]; then
    solver_cmd+=(--enforce-pass)
  fi

  if "${solver_cmd[@]}"; then
    :
  else
    solver_invocation_fail_count=$((solver_invocation_fail_count + 1))
  fi

  solver_status="$(cat "${solver_status_file}" 2>/dev/null || true)"
  solver_status="${solver_status:-UNKNOWN}"
  case "${solver_status}" in
    PASS)
      solver_pass_count=$((solver_pass_count + 1))
      ;;
    OPEN)
      solver_open_count=$((solver_open_count + 1))
      ;;
    *)
      solver_unknown_count=$((solver_unknown_count + 1))
      ;;
  esac

  if [[ "${ENFORCE_SOLVER_PASS}" -eq 1 && "${solver_status}" != "PASS" ]]; then
    solver_blocking_fail_count=$((solver_blocking_fail_count + 1))
  fi

  printf -- '- `%s` (solver: %s)\n' "${rel_path%.pf}.als" "${solver_status}" >> "${SUMMARY_FILE}"
done < <(find "${INPUT_DIR}" -type f -name "*.pf" | sort)

{
  echo
  echo "## Result"
  echo
  echo "- Generated artifacts: ${generated_count}"
  echo "- Generation failures: ${failed_count}"
  echo "- Solver PASS statuses: ${solver_pass_count}"
  echo "- Solver OPEN statuses: ${solver_open_count}"
  echo "- Solver UNKNOWN statuses: ${solver_unknown_count}"
  echo "- Solver invocation failures: ${solver_invocation_fail_count}"
  if [[ "${ENFORCE_SOLVER_PASS}" -eq 1 ]]; then
    echo "- Solver blocking failures: ${solver_blocking_fail_count}"
  fi
} >> "${SUMMARY_FILE}"

echo "Generated ${SUMMARY_FILE}"

if [[ "${failed_count}" -ne 0 ]]; then
  exit 1
fi

if [[ "${ENFORCE_SOLVER_PASS}" -eq 1 ]]; then
  if [[ "${solver_blocking_fail_count}" -ne 0 || "${solver_invocation_fail_count}" -ne 0 ]]; then
    exit 1
  fi
fi
