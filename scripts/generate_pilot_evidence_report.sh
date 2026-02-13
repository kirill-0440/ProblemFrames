#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/generate_pilot_evidence_report.sh [pilot_evidence.tsv] [output_dir]

Defaults:
  pilot_evidence.tsv = docs/adoption/pilot-evidence.tsv
  output_dir         = .ci-artifacts/adoption-pilot
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

INPUT_FILE="${1:-${REPO_ROOT}/docs/adoption/pilot-evidence.tsv}"
OUTPUT_DIR="${2:-${REPO_ROOT}/.ci-artifacts/adoption-pilot}"

if [[ ! -f "${INPUT_FILE}" ]]; then
  echo "Pilot evidence file not found: ${INPUT_FILE}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

MARKDOWN_FILE="${OUTPUT_DIR}/pilot-evidence.md"
JSON_FILE="${OUTPUT_DIR}/pilot-evidence.json"

stats_line="$(
  awk -F '\t' '
    NR == 1 { next }
    NF < 7 { next }
    {
      rows += 1
      base_cycles += $2
      post_cycles += $3
      base_minutes += $4
      post_minutes += $5
      base_defects += $6
      post_defects += $7
    }
    END {
      if (rows == 0) {
        print "0 0 0 0 0 0 0"
      } else {
        print rows, base_cycles / rows, post_cycles / rows, base_minutes / rows, post_minutes / rows, base_defects / rows, post_defects / rows
      }
    }
  ' "${INPUT_FILE}"
)"

read -r rows avg_base_cycles avg_post_cycles avg_base_minutes avg_post_minutes avg_base_defects avg_post_defects <<< "${stats_line}"

review_delta=$(awk -v b="${avg_base_cycles}" -v p="${avg_post_cycles}" 'BEGIN { printf "%.2f", b - p }')
minutes_delta=$(awk -v b="${avg_base_minutes}" -v p="${avg_post_minutes}" 'BEGIN { printf "%.2f", b - p }')
defects_delta=$(awk -v b="${avg_base_defects}" -v p="${avg_post_defects}" 'BEGIN { printf "%.2f", b - p }')

{
  echo "# Pilot Evidence Report"
  echo
  echo "- Source: \`${INPUT_FILE}\`"
  echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
  echo "- Teams sampled: ${rows}"
  echo
  echo "## Aggregated Metrics"
  echo
  echo "| Metric | Baseline | Post | Delta (improvement) |"
  echo "| --- | ---: | ---: | ---: |"
  echo "| Review cycles | ${avg_base_cycles} | ${avg_post_cycles} | ${review_delta} |"
  echo "| Model authoring time (minutes) | ${avg_base_minutes} | ${avg_post_minutes} | ${minutes_delta} |"
  echo "| Requirement defects | ${avg_base_defects} | ${avg_post_defects} | ${defects_delta} |"
} > "${MARKDOWN_FILE}"

{
  echo "{"
  echo "  \"source\": \"${INPUT_FILE#${REPO_ROOT}/}\","
  echo "  \"generated_at_utc\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\","
  echo "  \"teams\": ${rows},"
  echo "  \"review_cycles\": { \"baseline\": ${avg_base_cycles}, \"post\": ${avg_post_cycles}, \"delta\": ${review_delta} },"
  echo "  \"model_minutes\": { \"baseline\": ${avg_base_minutes}, \"post\": ${avg_post_minutes}, \"delta\": ${minutes_delta} },"
  echo "  \"requirement_defects\": { \"baseline\": ${avg_base_defects}, \"post\": ${avg_post_defects}, \"delta\": ${defects_delta} }"
  echo "}"
} > "${JSON_FILE}"

echo "Generated ${MARKDOWN_FILE}"
echo "Generated ${JSON_FILE}"
