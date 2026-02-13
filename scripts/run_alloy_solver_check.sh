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
  --closure-matrix-tsv <path>
                        Command-level closure matrix output path (TSV)
  --closure-matrix-md <path>
                        Command-level closure matrix output path (Markdown)
  --jar-path <path>     Explicit Alloy CLI jar path
  --solver <name>       Alloy solver id for `exec` (default: sat4j)
  --command <pattern>   Optional command selector for Alloy `exec`
  --repeat <n>          Number of solutions per command (default: 1)
  --expectations <path> Expectation manifest (model|command|SAT/UNSAT|note|required?)
  --enforce-pass        Exit non-zero when status is not PASS
  -h, --help            Show this help

Environment:
  PF_ALLOY_VERSION      Alloy release version (default: 6.2.0)
  PF_ALLOY_JAR_URL      Alloy CLI jar URL
  PF_ALLOY_CACHE_DIR    Cache directory for Alloy jar
  PF_ALLOY_JAR_SHA256   Expected SHA-256 for Alloy CLI jar (optional override)
  PF_ALLOY_CHECKSUM_MANIFEST
                        Versioned checksum manifest path (default: models/system/alloy_checksums.tsv)
USAGE
}

MODEL_FILE="${REPO_ROOT}/models/system/tool_spec.pf"
ALLOY_FILE=""
OUTPUT_DIR="${REPO_ROOT}/.ci-artifacts/alloy-solver"
REPORT_FILE=""
JSON_FILE=""
STATUS_FILE=""
CLOSURE_MATRIX_TSV=""
CLOSURE_MATRIX_MD=""
ALLOY_VERSION="${PF_ALLOY_VERSION:-6.2.0}"
ALLOY_JAR_URL="${PF_ALLOY_JAR_URL:-https://github.com/AlloyTools/org.alloytools.alloy/releases/download/v${ALLOY_VERSION}/org.alloytools.alloy.dist.jar}"
ALLOY_CACHE_DIR="${PF_ALLOY_CACHE_DIR:-${HOME}/.cache/problemframes/alloy}"
ALLOY_JAR_SHA256="${PF_ALLOY_JAR_SHA256:-}"
ALLOY_CHECKSUM_MANIFEST="${PF_ALLOY_CHECKSUM_MANIFEST:-${REPO_ROOT}/models/system/alloy_checksums.tsv}"
JAR_PATH=""
SOLVER_NAME="sat4j"
COMMAND_SELECTOR=""
REPEAT_COUNT=1
EXPECTATIONS_FILE=""
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
    --closure-matrix-tsv)
      CLOSURE_MATRIX_TSV="$2"
      shift 2
      ;;
    --closure-matrix-md)
      CLOSURE_MATRIX_MD="$2"
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
    --expectations)
      EXPECTATIONS_FILE="$2"
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
if [[ -n "${CLOSURE_MATRIX_TSV}" && "${CLOSURE_MATRIX_TSV}" != /* ]]; then
  CLOSURE_MATRIX_TSV="${REPO_ROOT}/${CLOSURE_MATRIX_TSV}"
fi
if [[ -n "${CLOSURE_MATRIX_MD}" && "${CLOSURE_MATRIX_MD}" != /* ]]; then
  CLOSURE_MATRIX_MD="${REPO_ROOT}/${CLOSURE_MATRIX_MD}"
fi
if [[ -n "${JAR_PATH}" && "${JAR_PATH}" != /* ]]; then
  JAR_PATH="${REPO_ROOT}/${JAR_PATH}"
fi
if [[ -n "${EXPECTATIONS_FILE}" && "${EXPECTATIONS_FILE}" != /* ]]; then
  EXPECTATIONS_FILE="${REPO_ROOT}/${EXPECTATIONS_FILE}"
fi
if [[ "${ALLOY_CHECKSUM_MANIFEST}" != /* ]]; then
  ALLOY_CHECKSUM_MANIFEST="${REPO_ROOT}/${ALLOY_CHECKSUM_MANIFEST}"
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
if [[ -z "${CLOSURE_MATRIX_TSV}" ]]; then
  CLOSURE_MATRIX_TSV="${OUTPUT_DIR}/alloy-solver-command-closure.tsv"
fi
if [[ -z "${CLOSURE_MATRIX_MD}" ]]; then
  CLOSURE_MATRIX_MD="${OUTPUT_DIR}/alloy-solver-command-closure.md"
fi

mkdir -p "$(dirname -- "${REPORT_FILE}")"
mkdir -p "$(dirname -- "${JSON_FILE}")"
mkdir -p "$(dirname -- "${STATUS_FILE}")"
mkdir -p "$(dirname -- "${CLOSURE_MATRIX_TSV}")"
mkdir -p "$(dirname -- "${CLOSURE_MATRIX_MD}")"

status="OPEN"
reason=""
command_count=0
solved_count=0
unsolved_count=0
unsolved_commands=""
solver_exit_code=0
expectation_mismatch_count=0
expectation_mismatch_commands=""
expectation_rules_used=0
expectation_summary=""
required_expectation_rules=0
required_expectation_rules_matched=0
required_expectation_rules_missing=0
required_expectation_missing_rules=""
model_rel_path=""
expectation_rule_count=0
expected_jar_sha256=""
actual_jar_sha256=""
checksum_source=""
checksum_verified=0

declare -a expectation_models=()
declare -a expectation_commands=()
declare -a expectation_verdicts=()
declare -a expectation_required=()
declare -a expectation_notes=()
declare -a expectation_matched=()
declare -a command_matrix_rows=()

exec_dir="${OUTPUT_DIR}/exec"
exec_stdout="${OUTPUT_DIR}/alloy-exec.stdout.log"
exec_stderr="${OUTPUT_DIR}/alloy-exec.stderr.log"

trim_field() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "${value}"
}

compute_sha256() {
  local file_path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file_path}" | awk '{print tolower($1)}'
    return 0
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file_path}" | awk '{print tolower($1)}'
    return 0
  fi
  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "${file_path}" | sed -E 's/^.*= *//; s/[[:space:]]+$//' | tr '[:upper:]' '[:lower:]'
    return 0
  fi
  return 1
}

load_expected_jar_sha256() {
  local manifest_version
  local manifest_sha
  local manifest_extra
  local normalized

  if [[ -n "${ALLOY_JAR_SHA256}" ]]; then
    normalized="$(trim_field "${ALLOY_JAR_SHA256}" | tr '[:upper:]' '[:lower:]')"
    expected_jar_sha256="${normalized}"
    checksum_source="env:PF_ALLOY_JAR_SHA256"
  elif [[ -f "${ALLOY_CHECKSUM_MANIFEST}" ]]; then
    while IFS='|' read -r manifest_version manifest_sha manifest_extra; do
      manifest_version="$(trim_field "${manifest_version:-}")"
      manifest_sha="$(trim_field "${manifest_sha:-}")"
      if [[ -z "${manifest_version}" || "${manifest_version:0:1}" == "#" ]]; then
        continue
      fi
      if [[ "${manifest_version}" == "${ALLOY_VERSION}" ]]; then
        expected_jar_sha256="$(tr '[:upper:]' '[:lower:]' <<< "${manifest_sha}")"
        checksum_source="${ALLOY_CHECKSUM_MANIFEST}:${ALLOY_VERSION}"
        break
      fi
    done < "${ALLOY_CHECKSUM_MANIFEST}"
  fi

  if [[ -z "${expected_jar_sha256}" ]]; then
    reason="missing expected Alloy jar checksum for version ${ALLOY_VERSION} (set PF_ALLOY_JAR_SHA256 or update ${ALLOY_CHECKSUM_MANIFEST})"
    return 1
  fi
  if [[ ! "${expected_jar_sha256}" =~ ^[0-9a-f]{64}$ ]]; then
    reason="invalid expected Alloy jar checksum '${expected_jar_sha256}'"
    return 1
  fi
  return 0
}

verify_jar_integrity() {
  local jar_path="$1"
  actual_jar_sha256="$(compute_sha256 "${jar_path}" 2>/dev/null || true)"
  if [[ -z "${actual_jar_sha256}" ]]; then
    reason="unable to compute Alloy jar SHA-256 (sha256sum/shasum/openssl not available)"
    return 1
  fi
  if [[ "${actual_jar_sha256}" != "${expected_jar_sha256}" ]]; then
    reason="alloy CLI jar checksum mismatch (expected ${expected_jar_sha256}, got ${actual_jar_sha256})"
    return 1
  fi
  checksum_verified=1
  return 0
}

normalize_required_flag() {
  local raw_flag="$1"
  local normalized
  normalized="$(trim_field "${raw_flag}")"
  normalized="${normalized,,}"
  case "${normalized}" in
    ""|"optional"|"false"|"0"|"no")
      printf '0'
      ;;
    "required"|"true"|"1"|"yes")
      printf '1'
      ;;
    *)
      printf 'invalid'
      ;;
  esac
}

expectation_model_matches_index() {
  local index="$1"
  local rule_model="${expectation_models[$index]}"
  [[ "${model_rel_path}" == ${rule_model} || "${MODEL_FILE}" == ${rule_model} ]]
}

load_expectation_rules() {
  local row_model
  local row_command
  local row_expected
  local row_note
  local row_required
  local expected_upper
  local required_flag

  if [[ -z "${EXPECTATIONS_FILE}" ]]; then
    return 0
  fi

  while IFS='|' read -r row_model row_command row_expected row_note row_required _; do
    row_model="$(trim_field "${row_model:-}")"
    row_command="$(trim_field "${row_command:-}")"
    row_expected="$(trim_field "${row_expected:-}")"
    row_note="$(trim_field "${row_note:-}")"
    row_required="$(trim_field "${row_required:-}")"

    if [[ -z "${row_model}" ]]; then
      continue
    fi
    if [[ "${row_model:0:1}" == "#" ]]; then
      continue
    fi
    if [[ -z "${row_command}" ]]; then
      row_command="*"
    fi
    if [[ -z "${row_expected}" ]]; then
      continue
    fi

    expected_upper="${row_expected^^}"
    if [[ "${expected_upper}" != "SAT" && "${expected_upper}" != "UNSAT" ]]; then
      reason="invalid expectation verdict '${row_expected}' for rule ${row_model}|${row_command}"
      return 0
    fi

    required_flag="$(normalize_required_flag "${row_required}")"
    if [[ "${required_flag}" == "invalid" ]]; then
      reason="invalid required flag '${row_required}' for rule ${row_model}|${row_command}"
      return 0
    fi

    expectation_models+=("${row_model}")
    expectation_commands+=("${row_command}")
    expectation_verdicts+=("${expected_upper}")
    expectation_required+=("${required_flag}")
    expectation_notes+=("${row_note}")
    expectation_matched+=("0")
  done < "${EXPECTATIONS_FILE}"

  expectation_rule_count="${#expectation_models[@]}"
}

resolve_expected_rule() {
  local command_name="$1"
  local idx

  if [[ -z "${EXPECTATIONS_FILE}" ]]; then
    printf '%s' ""
    return 0
  fi

  for idx in "${!expectation_models[@]}"; do
    if ! expectation_model_matches_index "${idx}"; then
      continue
    fi
    if [[ "${command_name}" == ${expectation_commands[$idx]} ]]; then
      printf '%s' "${idx}|${expectation_verdicts[$idx]}"
      return 0
    fi
  done

  printf '%s' ""
}

sanitize_matrix_field() {
  local value="$1"
  value="${value//$'\n'/ }"
  value="${value//$'\r'/ }"
  value="${value//|//}"
  printf '%s' "${value}"
}

write_command_matrix_outputs() {
  {
    echo "# model|command|expected|actual|status|rule|required|note"
    for row in "${command_matrix_rows[@]}"; do
      echo "${row}"
    done
  } > "${CLOSURE_MATRIX_TSV}"

  {
    echo "# Alloy Command Closure Matrix"
    echo
    echo "- Model source: \`${MODEL_FILE}\`"
    echo "- Status: \`${status}\`"
    echo
    echo "| Command | Expected | Actual | Status | Rule | Required | Note |"
    echo "| --- | --- | --- | --- | --- | --- | --- |"
    if [[ "${#command_matrix_rows[@]}" -eq 0 ]]; then
      echo "| - | - | - | no_commands | - | - | - |"
    else
      for row in "${command_matrix_rows[@]}"; do
        IFS='|' read -r _model_entry command_name expected actual row_status rule_label required_flag row_note <<< "${row}"
        required_text="optional"
        if [[ "${required_flag}" == "1" ]]; then
          required_text="required"
        fi
        if [[ -z "${row_note}" ]]; then
          row_note="-"
        fi
        echo "| \`${command_name}\` | ${expected} | ${actual} | ${row_status} | \`${rule_label}\` | ${required_text} | ${row_note} |"
      done
    fi
  } > "${CLOSURE_MATRIX_MD}"
}

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
    if [[ -n "${expected_jar_sha256}" ]]; then
      echo "- Alloy jar SHA-256 (expected): \`${expected_jar_sha256}\`"
    else
      echo "- Alloy jar SHA-256 (expected): \`<missing>\`"
    fi
    if [[ -n "${checksum_source}" ]]; then
      echo "- Alloy checksum source: \`${checksum_source}\`"
    else
      echo "- Alloy checksum source: \`<unset>\`"
    fi
    if [[ -n "${actual_jar_sha256}" ]]; then
      echo "- Alloy jar SHA-256 (actual): \`${actual_jar_sha256}\`"
    else
      echo "- Alloy jar SHA-256 (actual): \`<unavailable>\`"
    fi
    if [[ "${checksum_verified}" -eq 1 ]]; then
      echo "- Alloy jar integrity verification: \`PASS\`"
    else
      echo "- Alloy jar integrity verification: \`FAIL\`"
    fi
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
    if [[ -n "${EXPECTATIONS_FILE}" ]]; then
      echo "- Expectations file: \`${EXPECTATIONS_FILE}\`"
      echo "- Expectation rules total: \`${expectation_rule_count}\`"
      echo "- Expectation rules used: \`${expectation_rules_used}\`"
      echo "- Expectation mismatches: \`${expectation_mismatch_count}\`"
      echo "- Required expectation rules: \`${required_expectation_rules}\`"
      echo "- Required rules matched: \`${required_expectation_rules_matched}\`"
      echo "- Required rules missing: \`${required_expectation_rules_missing}\`"
      if [[ -n "${expectation_summary}" ]]; then
        echo "- Expectation summary: ${expectation_summary}"
      fi
      if [[ -n "${expectation_mismatch_commands}" ]]; then
        echo "- Expectation mismatch commands: \`${expectation_mismatch_commands}\`"
      fi
      if [[ -n "${required_expectation_missing_rules}" ]]; then
        echo "- Missing required rules: \`${required_expectation_missing_rules}\`"
      fi
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
    echo "- \`${CLOSURE_MATRIX_TSV}\`"
    echo "- \`${CLOSURE_MATRIX_MD}\`"
  } > "${REPORT_FILE}"

  {
    echo "{"
    echo "  \"status\": \"${status}\","
    echo "  \"reason\": \"${reason}\","
    echo "  \"model\": \"${MODEL_FILE}\","
    echo "  \"alloy_model\": \"${ALLOY_FILE}\","
    echo "  \"jar\": \"${JAR_PATH}\","
    echo "  \"alloy_version\": \"${ALLOY_VERSION}\","
    echo "  \"alloy_checksum_expected\": \"${expected_jar_sha256}\","
    echo "  \"alloy_checksum_source\": \"${checksum_source}\","
    echo "  \"alloy_checksum_actual\": \"${actual_jar_sha256}\","
    echo "  \"alloy_checksum_verified\": ${checksum_verified},"
    echo "  \"solver\": \"${SOLVER_NAME}\","
    if [[ -n "${COMMAND_SELECTOR}" ]]; then
      echo "  \"command_selector\": \"${COMMAND_SELECTOR}\","
    else
      echo "  \"command_selector\": \"\","
    fi
    if [[ -n "${EXPECTATIONS_FILE}" ]]; then
      echo "  \"expectations_file\": \"${EXPECTATIONS_FILE}\","
    else
      echo "  \"expectations_file\": \"\","
    fi
    echo "  \"repeat\": ${REPEAT_COUNT},"
    echo "  \"solver_exit_code\": ${solver_exit_code},"
    echo "  \"expectation_rule_count\": ${expectation_rule_count},"
    echo "  \"expectation_rules_used\": ${expectation_rules_used},"
    echo "  \"expectation_mismatch_count\": ${expectation_mismatch_count},"
    echo "  \"required_expectation_rules\": ${required_expectation_rules},"
    echo "  \"required_expectation_rules_matched\": ${required_expectation_rules_matched},"
    echo "  \"required_expectation_rules_missing\": ${required_expectation_rules_missing},"
    if [[ -n "${expectation_summary}" ]]; then
      echo "  \"expectation_summary\": \"${expectation_summary}\","
    else
      echo "  \"expectation_summary\": \"\","
    fi
    if [[ -n "${expectation_mismatch_commands}" ]]; then
      echo "  \"expectation_mismatch_commands\": \"${expectation_mismatch_commands}\","
    else
      echo "  \"expectation_mismatch_commands\": \"\","
    fi
    if [[ -n "${required_expectation_missing_rules}" ]]; then
      echo "  \"required_expectation_missing_rules\": \"${required_expectation_missing_rules}\","
    else
      echo "  \"required_expectation_missing_rules\": \"\","
    fi
    echo "  \"command_count\": ${command_count},"
    echo "  \"solved_count\": ${solved_count},"
    echo "  \"unsolved_count\": ${unsolved_count},"
    if [[ -n "${unsolved_commands}" ]]; then
      echo "  \"unsolved_commands\": \"${unsolved_commands}\""
    else
      echo "  \"unsolved_commands\": \"\""
    fi
    echo ","
    echo "  \"closure_matrix_tsv\": \"${CLOSURE_MATRIX_TSV}\","
    echo "  \"closure_matrix_md\": \"${CLOSURE_MATRIX_MD}\""
    echo "}"
  } > "${JSON_FILE}"

  write_command_matrix_outputs

  echo "${status}" > "${STATUS_FILE}"
  echo "Generated ${REPORT_FILE}"
  echo "Generated ${JSON_FILE}"
  echo "Generated ${CLOSURE_MATRIX_TSV}"
  echo "Generated ${CLOSURE_MATRIX_MD}"
}

if [[ ! -f "${MODEL_FILE}" ]]; then
  reason="model file is missing"
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

if [[ "${MODEL_FILE}" == "${REPO_ROOT}/"* ]]; then
  model_rel_path="${MODEL_FILE#${REPO_ROOT}/}"
else
  model_rel_path="${MODEL_FILE}"
fi

if [[ -n "${EXPECTATIONS_FILE}" && ! -f "${EXPECTATIONS_FILE}" ]]; then
  reason="expectations file is missing"
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

load_expectation_rules
if [[ -n "${reason}" ]]; then
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

if ! load_expected_jar_sha256; then
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
fi

downloaded_jar=0
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
  downloaded_jar=1
fi

if ! verify_jar_integrity "${JAR_PATH}"; then
  if [[ "${downloaded_jar}" -eq 1 ]]; then
    rm -f "${JAR_PATH}" || true
  fi
  write_outputs
  if [[ "${ENFORCE_PASS}" -eq 1 ]]; then
    exit 1
  fi
  exit 0
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
  if command -v jq >/dev/null 2>&1; then
    mapfile -t command_results < <(
      jq -r '.commands | to_entries[] | [.key, (if ((.value.solution | length) > 0) then "SAT" else "UNSAT" end)] | @tsv' "${receipt_file}"
    )
    command_count="${#command_results[@]}"
    solved_count=0
    unsolved_count=0
    unsolved_commands=""
    expectation_mismatch_count=0
    expectation_mismatch_commands=""
    expectation_rules_used=0

    for command_result in "${command_results[@]}"; do
      command_name="${command_result%%$'\t'*}"
      actual_verdict="${command_result#*$'\t'}"

      expected_verdict="SAT"
      matched_rule_id="<default>"
      matched_rule_required="0"
      matched_rule_note="default SAT expectation"
      if [[ -n "${EXPECTATIONS_FILE}" ]]; then
        resolved_rule="$(resolve_expected_rule "${command_name}")"
        if [[ -n "${resolved_rule}" ]]; then
          resolved_rule_index="${resolved_rule%%|*}"
          resolved_rule_verdict="${resolved_rule#*|}"
          expected_verdict="${resolved_rule_verdict}"
          expectation_rules_used=$((expectation_rules_used + 1))
          expectation_matched[$resolved_rule_index]="1"
          matched_rule_id="$(sanitize_matrix_field "${expectation_models[$resolved_rule_index]}::${expectation_commands[$resolved_rule_index]}")"
          matched_rule_required="${expectation_required[$resolved_rule_index]}"
          matched_rule_note="$(sanitize_matrix_field "${expectation_notes[$resolved_rule_index]}")"
        fi
      fi

      if [[ "${actual_verdict}" == "SAT" ]]; then
        solved_count=$((solved_count + 1))
      else
        unsolved_count=$((unsolved_count + 1))
        if [[ -z "${unsolved_commands}" ]]; then
          unsolved_commands="${command_name}"
        else
          unsolved_commands="${unsolved_commands},${command_name}"
        fi
      fi

      if [[ "${actual_verdict}" != "${expected_verdict}" ]]; then
        expectation_mismatch_count=$((expectation_mismatch_count + 1))
        mismatch_label="${command_name}:${expected_verdict}/${actual_verdict}"
        if [[ -z "${expectation_mismatch_commands}" ]]; then
          expectation_mismatch_commands="${mismatch_label}"
        else
          expectation_mismatch_commands="${expectation_mismatch_commands},${mismatch_label}"
        fi
      fi

      command_row_status="MATCH"
      if [[ "${actual_verdict}" != "${expected_verdict}" ]]; then
        command_row_status="MISMATCH"
      fi
      command_matrix_rows+=(
        "$(sanitize_matrix_field "${model_rel_path}")|$(sanitize_matrix_field "${command_name}")|${expected_verdict}|${actual_verdict}|${command_row_status}|${matched_rule_id}|${matched_rule_required}|${matched_rule_note}"
      )
    done

    if [[ -n "${EXPECTATIONS_FILE}" ]]; then
      required_expectation_rules=0
      required_expectation_rules_matched=0
      required_expectation_rules_missing=0
      required_expectation_missing_rules=""

      for idx in "${!expectation_models[@]}"; do
        if ! expectation_model_matches_index "${idx}"; then
          continue
        fi
        if [[ "${expectation_required[$idx]}" == "1" ]]; then
          required_expectation_rules=$((required_expectation_rules + 1))
          if [[ "${expectation_matched[$idx]}" == "1" ]]; then
            required_expectation_rules_matched=$((required_expectation_rules_matched + 1))
          else
            required_expectation_rules_missing=$((required_expectation_rules_missing + 1))
            missing_label="${expectation_models[$idx]}::${expectation_commands[$idx]}"
            if [[ -z "${required_expectation_missing_rules}" ]]; then
              required_expectation_missing_rules="${missing_label}"
            else
              required_expectation_missing_rules="${required_expectation_missing_rules},${missing_label}"
            fi
          fi
        fi
      done

      expectation_summary="default=SAT; matched_rules=${expectation_rules_used}; required=${required_expectation_rules}; required_matched=${required_expectation_rules_matched}"
    fi
  else
    command_count="$(
      {
        grep -Eo '"overall"[[:space:]]*:' "${receipt_file}" || true
      } | wc -l | tr -d ' '
    )"
    unsolved_count="$(
      {
        grep -Eo '"solution"[[:space:]]*:[[:space:]]*\\[\\]' "${receipt_file}" || true
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
    if [[ -n "${EXPECTATIONS_FILE}" ]]; then
      reason="jq is required when --expectations is used"
    fi
  fi

  if [[ -z "${reason}" ]]; then
    if [[ "${command_count}" -eq 0 ]]; then
      reason="no executable Alloy commands were found"
    elif [[ "${expectation_mismatch_count}" -gt 0 ]]; then
      reason="${expectation_mismatch_count} command(s) violated SAT/UNSAT expectations"
    elif [[ "${required_expectation_rules_missing}" -gt 0 ]]; then
      reason="${required_expectation_rules_missing} required expectation rule(s) were not matched by executed commands"
    else
      status="PASS"
      if [[ "${unsolved_count}" -gt 0 ]]; then
        reason="all commands matched expectations (includes expected UNSAT)"
      else
        reason="all commands produced SAT as expected"
      fi
    fi
  fi
fi

write_outputs

if [[ "${ENFORCE_PASS}" -eq 1 && "${status}" != "PASS" ]]; then
  echo "Alloy solver status is ${status}; expected PASS." >&2
  exit 1
fi
