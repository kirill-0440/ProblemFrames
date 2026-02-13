#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_alloy_solver_check.sh [options]

Options:
  --model <path>        PF model path (default: models/system/tool_spec.pf)
  --alloy-file <path>   Existing Alloy model (.als). If omitted, generated from --model.
  --output-dir <dir>    Output directory (default: .ci-artifacts/alloy-solver)
  --report <path>       Markdown report output path
  --json <path>         JSON summary output path
  --status-file <path>  Status output path (PASS/OPEN)
  --jar-path <path>     Explicit Alloy CLI jar path
  --solver <name>       Alloy solver id for `exec` (default: sat4j)
  --command <pattern>   Optional command selector for Alloy `exec`
  --repeat <n>          Number of solutions per command (default: 1)
  --enforce-pass        Exit non-zero when status is not PASS
  -h, --help            Show this help

Environment:
  PF_ALLOY_VERSION      Alloy release version (default: 6.2.0)
  PF_ALLOY_JAR_URL      Alloy CLI jar URL
  PF_ALLOY_CACHE_DIR    Cache directory for Alloy jar
USAGE
}

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
ALLOY_FILE=""
OUTPUT_DIR="${REPO_ROOT}/.ci-artifacts/alloy-solver"
REPORT_FILE=""
JSON_FILE=""
STATUS_FILE=""
ALLOY_VERSION="${PF_ALLOY_VERSION:-6.2.0}"
ALLOY_JAR_URL="${PF_ALLOY_JAR_URL:-https://github.com/AlloyTools/org.alloytools.alloy/releases/download/v${ALLOY_VERSION}/org.alloytools.alloy.dist.jar}"
ALLOY_CACHE_DIR="${PF_ALLOY_CACHE_DIR:-${HOME}/.cache/problemframes/alloy}"
JAR_PATH=""
SOLVER_NAME="sat4j"
COMMAND_SELECTOR=""
REPEAT_COUNT=1
ENFORCE_PASS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)
      MODEL_FILE="$2"
      shift 2
      ;;
    --alloy-file)
      ALLOY_FILE="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --report)
      REPORT_FILE="$2"
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
    --jar-path)
      JAR_PATH="$2"
      shift 2
      ;;
    --solver)
      SOLVER_NAME="$2"
      shift 2
      ;;
    --command)
      COMMAND_SELECTOR="$2"
      shift 2
      ;;
    --repeat)
      REPEAT_COUNT="$2"
      shift 2
      ;;
    --enforce-pass)
      ENFORCE_PASS=1
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
if [[ -n "${ALLOY_FILE}" && "${ALLOY_FILE}" != /* ]]; then
  ALLOY_FILE="${REPO_ROOT}/${ALLOY_FILE}"
fi
if [[ "${OUTPUT_DIR}" != /* ]]; then
  OUTPUT_DIR="${REPO_ROOT}/${OUTPUT_DIR}"
fi
if [[ -n "${REPORT_FILE}" && "${REPORT_FILE}" != /* ]]; then
  REPORT_FILE="${REPO_ROOT}/${REPORT_FILE}"
fi
if [[ -n "${JSON_FILE}" && "${JSON_FILE}" != /* ]]; then
  JSON_FILE="${REPO_ROOT}/${JSON_FILE}"
fi
if [[ -n "${STATUS_FILE}" && "${STATUS_FILE}" != /* ]]; then
  STATUS_FILE="${REPO_ROOT}/${STATUS_FILE}"
fi
if [[ -n "${JAR_PATH}" && "${JAR_PATH}" != /* ]]; then
  JAR_PATH="${REPO_ROOT}/${JAR_PATH}"
fi

if ! [[ "${REPEAT_COUNT}" =~ ^[0-9]+$ ]] || [[ "${REPEAT_COUNT}" -lt 1 ]]; then
  echo "Invalid --repeat value: ${REPEAT_COUNT}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

if [[ -z "${REPORT_FILE}" ]]; then
  REPORT_FILE="${OUTPUT_DIR}/alloy-solver-check.md"
fi
if [[ -z "${JSON_FILE}" ]]; then
  JSON_FILE="${OUTPUT_DIR}/alloy-solver-check.json"
fi
if [[ -z "${STATUS_FILE}" ]]; then
  STATUS_FILE="${OUTPUT_DIR}/alloy-solver-check.status"
fi

mkdir -p "$(dirname -- "${REPORT_FILE}")"
mkdir -p "$(dirname -- "${JSON_FILE}")"
mkdir -p "$(dirname -- "${STATUS_FILE}")"

status="OPEN"
reason=""
command_count=0
solved_count=0
unsolved_count=0
unsolved_commands=""
solver_exit_code=0

exec_dir="${OUTPUT_DIR}/exec"
exec_stdout="${OUTPUT_DIR}/alloy-exec.stdout.log"
exec_stderr="${OUTPUT_DIR}/alloy-exec.stderr.log"

write_outputs() {
  local generated_utc
  generated_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  {
    echo "# Alloy Solver Check"
    echo
    echo "- Generated (UTC): \`${generated_utc}\`"
    echo "- Model source: \`${MODEL_FILE}\`"
    echo "- Alloy model: \`${ALLOY_FILE}\`"
    echo "- Alloy CLI jar: \`${JAR_PATH}\`"
    echo "- Alloy version: \`${ALLOY_VERSION}\`"
    echo "- Solver: \`${SOLVER_NAME}\`"
    if [[ -n "${COMMAND_SELECTOR}" ]]; then
      echo "- Command selector: \`${COMMAND_SELECTOR}\`"
    else
      echo "- Command selector: \`<all>\`"
    fi
    echo "- Repeat: \`${REPEAT_COUNT}\`"
    echo "- Solver exit code: \`${solver_exit_code}\`"
    echo "- Status: \`${status}\`"
    if [[ -n "${reason}" ]]; then
      echo "- Reason: ${reason}"
    fi
    echo "- Commands total: \`${command_count}\`"
    echo "- Commands solved: \`${solved_count}\`"
    echo "- Commands unsolved: \`${unsolved_count}\`"
    if [[ -n "${unsolved_commands}" ]]; then
      echo "- Unsolved commands: \`${unsolved_commands}\`"
    fi
    echo
    echo "## Artifacts"
    echo
    echo "- \`${exec_dir}/receipt.json\`"
    echo "- \`${exec_stdout}\`"
    echo "- \`${exec_stderr}\`"
  } > "${REPORT_FILE}"

  {
    echo "{"
    echo "  \"status\": \"${status}\","
    echo "  \"reason\": \"${reason}\","
    echo "  \"model\": \"${MODEL_FILE}\","
    echo "  \"alloy_model\": \"${ALLOY_FILE}\","
    echo "  \"jar\": \"${JAR_PATH}\","
    echo "  \"alloy_version\": \"${ALLOY_VERSION}\","
    echo "  \"solver\": \"${SOLVER_NAME}\","
    if [[ -n "${COMMAND_SELECTOR}" ]]; then
      echo "  \"command_selector\": \"${COMMAND_SELECTOR}\","
    else
      echo "  \"command_selector\": \"\","
    fi
    echo "  \"repeat\": ${REPEAT_COUNT},"
    echo "  \"solver_exit_code\": ${solver_exit_code},"
    echo "  \"command_count\": ${command_count},"
    echo "  \"solved_count\": ${solved_count},"
    echo "  \"unsolved_count\": ${unsolved_count},"
    if [[ -n "${unsolved_commands}" ]]; then
      echo "  \"unsolved_commands\": \"${unsolved_commands}\""
    else
      echo "  \"unsolved_commands\": \"\""
    fi
    echo "}"
  } > "${JSON_FILE}"

  echo "${status}" > "${STATUS_FILE}"
  echo "Generated ${REPORT_FILE}"
  echo "Generated ${JSON_FILE}"
}

if [[ ! -f "${MODEL_FILE}" ]]; then
  reason="model file is missing"
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

if ! command -v java >/dev/null 2>&1; then
  reason="java is not available in PATH"
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

if [[ -z "${JAR_PATH}" ]]; then
  JAR_PATH="${ALLOY_CACHE_DIR}/org.alloytools.alloy.dist-${ALLOY_VERSION}.jar"
fi

if [[ ! -f "${JAR_PATH}" ]]; then
  mkdir -p "${ALLOY_CACHE_DIR}"
  if ! curl -fsSL -L --retry 3 --retry-delay 2 -o "${JAR_PATH}" "${ALLOY_JAR_URL}"; then
    reason="failed to download Alloy CLI jar"
    write_outputs
    if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
      exit 1
    fi
    exit 0
  fi
fi

if ! java -jar "${JAR_PATH}" help exec >/dev/null 2>&1; then
  reason="alloy CLI jar is not executable"
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

if [[ -z "${ALLOY_FILE}" ]]; then
  ALLOY_FILE="${OUTPUT_DIR}/model.als"
  cargo run -p pf_dsl -- "${MODEL_FILE}" --alloy > "${ALLOY_FILE}"
fi

if [[ ! -f "${ALLOY_FILE}" ]]; then
  reason="alloy model file is missing"
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

mkdir -p "${exec_dir}"

exec_args=(
  exec
  --quiet
  --force
  --solver "${SOLVER_NAME}"
  --repeat "${REPEAT_COUNT}"
  --output "${exec_dir}"
  --type json
)
if [[ -n "${COMMAND_SELECTOR}" ]]; then
  exec_args+=(--command "${COMMAND_SELECTOR}")
fi
exec_args+=("${ALLOY_FILE}")

if java -jar "${JAR_PATH}" "${exec_args[@]}" > "${exec_stdout}" 2> "${exec_stderr}"; then
  solver_exit_code=0
else
  solver_exit_code=$?
fi

receipt_file="${exec_dir}/receipt.json"

if [[ "${solver_exit_code}" -ne 0 ]]; then
  reason="alloy exec command failed"
elif [[ ! -f "${receipt_file}" ]]; then
  reason="receipt.json is missing after alloy exec"
else
  command_count="$(
    {
      grep -o '"source":"' "${receipt_file}" || true
    } | wc -l | tr -d ' '
  )"
  unsolved_count="$(
    {
      grep -o '"solution":\\[\\]' "${receipt_file}" || true
    } | wc -l | tr -d ' '
  )"

  if [[ -z "${command_count}" ]]; then
    command_count=0
  fi
  if [[ -z "${unsolved_count}" ]]; then
    unsolved_count=0
  fi
  if [[ "${command_count}" -lt "${unsolved_count}" ]]; then
    unsolved_count="${command_count}"
  fi

  solved_count=$((command_count - unsolved_count))

  if [[ "${command_count}" -eq 0 ]]; then
    reason="no executable Alloy commands were found"
  elif [[ "${unsolved_count}" -gt 0 ]]; then
    reason="${unsolved_count} command(s) returned no solution"
    if command -v jq >/dev/null 2>&1; then
      unsolved_commands="$(
        jq -r '.commands | to_entries[] | select((.value.solution | length) == 0) | .key' "${receipt_file}" \
          | paste -sd ',' - \
          || true
      )"
      unsolved_commands="${unsolved_commands:-}"
    fi
  else
    status="PASS"
    reason="all commands produced at least one solution"
  fi
fi

write_outputs

if [[ "${ENFORCE_PASS}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Alloy solver status is ${status}; expected PASS." >&2
  exit 1
fi
