#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

INPUT_DIR="${REPO_ROOT}/crates/pf_dsl/dogfooding"
OUTPUT_DIR="${1:-${REPO_ROOT}/docs/formal-backend}"
SUMMARY_FILE="${OUTPUT_DIR}/formal-backend-summary.md"

if [[ ! -d "${INPUT_DIR}" ]]; then
  echo "Dogfooding directory not found: ${INPUT_DIR}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

generated_count=0
failed_count=0

{
  echo "# Formal Backend Check (Alloy)"
  echo
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Source models: \`${INPUT_DIR}\`"
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
    printf -- '- `%s`\n' "${rel_path%.pf}.als" >> "${SUMMARY_FILE}"
    echo "Generated ${out_file}"
  else
    failed_count=$((failed_count + 1))
    printf -- '- `%s` (generation failed)\n' "${rel_path%.pf}.als" >> "${SUMMARY_FILE}"
    echo "Failed to generate ${out_file}" >&2
  fi
done < <(find "${INPUT_DIR}" -type f -name "*.pf" | sort)

{
  echo
  echo "## Result"
  echo
  echo "- Generated artifacts: ${generated_count}"
  echo "- Generation failures: ${failed_count}"
  echo "- Solver execution: skipped (informational translator stage only)"
} >> "${SUMMARY_FILE}"

echo "Generated ${SUMMARY_FILE}"

if [[ "${failed_count}" -ne 0 ]]; then
  exit 1
fi
