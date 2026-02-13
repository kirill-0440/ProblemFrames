#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_pf_quality_gate.sh [options] <model.pf> [more models...]

Options:
  --allow-open-closure        Do not fail if decomposition closure status is FAIL.
  --allow-open-concern-coverage
                              Do not fail if concern coverage status is FAIL.
  --impact <selectors>        Impact seeds for traceability export (e.g. requirement:R1,domain:D1).
  --impact-hops <n>           Max hops for impact traversal in traceability export.
  -h, --help                  Show this help.

Output:
  .ci-artifacts/pf-quality-gate/<model-key>/
USAGE
}

allow_open_closure=0
allow_open_concern_coverage=0
impact_selectors=""
impact_hops=""
models=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --allow-open-closure)
      allow_open_closure=1
      shift
      ;;
    --allow-open-concern-coverage)
      allow_open_concern_coverage=1
      shift
      ;;
    --impact)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --impact" >&2
        exit 1
      fi
      impact_selectors="$2"
      shift 2
      ;;
    --impact-hops)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --impact-hops" >&2
        exit 1
      fi
      impact_hops="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
    *)
      models+=("$1")
      shift
      ;;
  esac
done

if [[ ${#models[@]} -eq 0 ]]; then
  usage
  exit 1
fi

output_root="${REPO_ROOT}/.ci-artifacts/pf-quality-gate"
mkdir -p "${output_root}"

failure_count=0

for model in "${models[@]}"; do
  if [[ ! -f "${model}" ]]; then
    echo "Model not found: ${model}" >&2
    failure_count=$((failure_count + 1))
    continue
  fi

  abs_model="$(cd -- "$(dirname -- "${model}")" && pwd)/$(basename -- "${model}")"
  if [[ "${abs_model}" == "${REPO_ROOT}/"* ]]; then
    model_key="${abs_model#${REPO_ROOT}/}"
  else
    model_key="$(basename -- "${abs_model}")"
  fi
  model_key="${model_key%.pf}"
  model_key="${model_key//\//__}"

  model_output_dir="${output_root}/${model_key}"
  mkdir -p "${model_output_dir}"

  report_file="${model_output_dir}/report.md"
  decomposition_file="${model_output_dir}/decomposition-closure.md"
  obligations_file="${model_output_dir}/obligations.md"
  concern_coverage_file="${model_output_dir}/concern-coverage.md"
  alloy_file="${model_output_dir}/model.als"
  traceability_md_file="${model_output_dir}/traceability.md"
  traceability_csv_file="${model_output_dir}/traceability.csv"
  wrspm_file="${model_output_dir}/wrspm.md"
  wrspm_json_file="${model_output_dir}/wrspm.json"
  summary_file="${model_output_dir}/summary.md"

  traceability_args=()
  if [[ -n "${impact_selectors}" ]]; then
    traceability_args+=("--impact=${impact_selectors}")
  fi
  if [[ -n "${impact_hops}" ]]; then
    traceability_args+=("--impact-hops=${impact_hops}")
  fi

  cargo run -p pf_dsl -- "${model}" --report > "${report_file}"
  cargo run -p pf_dsl -- "${model}" --decomposition-closure > "${decomposition_file}"
  cargo run -p pf_dsl -- "${model}" --obligations > "${obligations_file}"
  cargo run -p pf_dsl -- "${model}" --concern-coverage > "${concern_coverage_file}"
  cargo run -p pf_dsl -- "${model}" --alloy > "${alloy_file}"
  cargo run -p pf_dsl -- "${model}" --traceability-md "${traceability_args[@]}" > "${traceability_md_file}"
  cargo run -p pf_dsl -- "${model}" --traceability-csv "${traceability_args[@]}" > "${traceability_csv_file}"
  cargo run -p pf_dsl -- "${model}" --wrspm-report > "${wrspm_file}"
  cargo run -p pf_dsl -- "${model}" --wrspm-json > "${wrspm_json_file}"

  closure_status="$(
    grep -E "^- Closure status: " "${decomposition_file}" \
      | sed -e 's/^- Closure status: //'
  )"
  closure_status="${closure_status:-UNKNOWN}"
  concern_coverage_status="$(
    grep -E "^- Concern coverage status: " "${concern_coverage_file}" \
      | sed -e 's/^- Concern coverage status: //'
  )"
  concern_coverage_status="${concern_coverage_status:-UNKNOWN}"

  {
    echo "# PF Quality Gate Summary"
    echo
    echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
    echo "- Model: \`${model}\`"
    echo "- Decomposition closure status: \`${closure_status}\`"
    echo "- Concern coverage status: \`${concern_coverage_status}\`"
    echo
    echo "## Artifacts"
    echo
    echo "- \`report.md\`"
    echo "- \`decomposition-closure.md\`"
    echo "- \`obligations.md\`"
    echo "- \`concern-coverage.md\`"
    echo "- \`model.als\`"
    echo "- \`traceability.md\`"
    echo "- \`traceability.csv\`"
    echo "- \`wrspm.md\`"
    echo "- \`wrspm.json\`"
  } > "${summary_file}"

  echo "Generated gate artifacts for ${model} -> ${model_output_dir}"

  if [[ "${closure_status}" != "PASS" && "${allow_open_closure}" -eq 0 ]]; then
    echo "Decomposition closure failed for ${model}; re-run with --allow-open-closure to override." >&2
    failure_count=$((failure_count + 1))
  fi
  if [[ "${concern_coverage_status}" != "PASS" && "${allow_open_concern_coverage}" -eq 0 ]]; then
    echo "Concern coverage failed for ${model}; re-run with --allow-open-concern-coverage to override." >&2
    failure_count=$((failure_count + 1))
  fi
done

if [[ "${failure_count}" -ne 0 ]]; then
  exit 1
fi
