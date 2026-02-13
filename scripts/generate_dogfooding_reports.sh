#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

INPUT_DIR="${REPO_ROOT}/models/dogfooding"
OUTPUT_DIR="${1:-${REPO_ROOT}/docs/dogfooding-reports}"

if [[ ! -d "${INPUT_DIR}" ]]; then
  echo "Dogfooding directory not found: ${INPUT_DIR}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

while IFS= read -r model; do
  rel_path="${model#${INPUT_DIR}/}"
  out_file="${OUTPUT_DIR}/${rel_path%.pf}.md"

  mkdir -p "$(dirname -- "${out_file}")"

  {
    echo "<!-- Generated from ${rel_path}. Do not edit manually. -->"
    echo
    cargo run -p pf_dsl -- "${model}" --report
  } > "${out_file}"

  echo "Generated ${out_file}"
done < <(find "${INPUT_DIR}" -type f -name "*.pf" | sort)
