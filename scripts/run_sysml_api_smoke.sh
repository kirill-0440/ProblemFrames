#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

OUTPUT_DIR="${1:-${REPO_ROOT}/.ci-artifacts/sysml-api}"
OUTPUT_FILE="${OUTPUT_DIR}/smoke.json"

mkdir -p "${OUTPUT_DIR}"

endpoint="${PF_SYSML_API_ENDPOINT:-}"
args=(smoke --dry-run)
if [[ -n "${endpoint}" ]]; then
  args+=("--endpoint=${endpoint}")
fi

cargo run -p pf_sysml_api -- "${args[@]}" > "${OUTPUT_FILE}"

echo "Generated ${OUTPUT_FILE}"
