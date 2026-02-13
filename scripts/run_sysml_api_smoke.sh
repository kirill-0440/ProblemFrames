#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_sysml_api_smoke.sh [output_dir] [--gated]

Environment:
  PF_SYSML_API_ENDPOINT         Optional endpoint passed to smoke command.
  PF_SYSML_API_SMOKE_ENABLED    When --gated is used, run only if this is "1".
USAGE
}

OUTPUT_DIR="${REPO_ROOT}/.ci-artifacts/sysml-api"
gated=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --gated)
      gated=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [[ "${OUTPUT_DIR}" != "${REPO_ROOT}/.ci-artifacts/sysml-api" ]]; then
        echo "Unexpected argument: $1" >&2
        usage
        exit 1
      fi
      OUTPUT_DIR="$1"
      shift
      ;;
  esac
done

OUTPUT_FILE="${OUTPUT_DIR}/smoke.json"
LOG_FILE="${OUTPUT_DIR}/smoke.log"

mkdir -p "${OUTPUT_DIR}"

endpoint="${PF_SYSML_API_ENDPOINT:-}"
args=(smoke --dry-run)
if [[ -n "${endpoint}" ]]; then
  args+=("--endpoint=${endpoint}")
fi

if [[ "${gated}" -eq 1 && "${PF_SYSML_API_SMOKE_ENABLED:-0}" != "1" ]]; then
  cat > "${OUTPUT_FILE}" <<'EOF'
{
  "status": "SKIPPED",
  "mode": "gated",
  "message": "PF_SYSML_API_SMOKE_ENABLED is not set to 1"
}
EOF
  {
    echo "SysML API smoke skipped by gate."
    echo "Set PF_SYSML_API_SMOKE_ENABLED=1 to execute smoke run."
  } > "${LOG_FILE}"
  echo "Generated ${OUTPUT_FILE}"
  echo "Generated ${LOG_FILE}"
  exit 0
fi

{
  echo "Running pf_sysml_api smoke..."
  echo "Command: cargo run -p pf_sysml_api -- ${args[*]}"
} > "${LOG_FILE}"

cargo run -p pf_sysml_api -- "${args[@]}" > "${OUTPUT_FILE}" 2>> "${LOG_FILE}"

echo "Generated ${OUTPUT_FILE}"
echo "Generated ${LOG_FILE}"
