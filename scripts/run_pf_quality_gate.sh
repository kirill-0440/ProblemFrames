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
  --enforce-implementation-trace
                              Fail if implementation trace status is not PASS.
  --implementation-policy <path>
                              Policy env file for staged implementation-trace enforcement.
  --enforce-implementation-policy
                              Fail if implementation-trace policy status is not PASS.
  --min-lean-formalized-args <n>
                              Minimum number of formalized Lean correctness arguments (default: 0).
  --enforce-formal-track      Fail on formal-track statuses (Lean coverage and formal closure).
  --impact <selectors>        Impact seeds for traceability export (e.g. requirement:R1,domain:D1).
  --impact-hops <n>           Max hops for impact traversal in traceability export.
  -h, --help                  Show this help.

Output:
  .ci-artifacts/pf-quality-gate/<model-key>/
USAGE
}

allow_open_closure=0
allow_open_concern_coverage=0
enforce_implementation_trace=0
implementation_policy_path=""
enforce_implementation_policy=0
enforce_formal_track="${PF_FORMAL_TRACK_BLOCKING:-0}"
min_lean_formalized_args=0
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
    --enforce-implementation-trace)
      enforce_implementation_trace=1
      shift
      ;;
    --implementation-policy)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --implementation-policy" >&2
        exit 1
      fi
      implementation_policy_path="$2"
      shift 2
      ;;
    --enforce-implementation-policy)
      enforce_implementation_policy=1
      shift
      ;;
    --min-lean-formalized-args)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --min-lean-formalized-args" >&2
        exit 1
      fi
      min_lean_formalized_args="$2"
      shift 2
      ;;
    --enforce-formal-track)
      enforce_formal_track=1
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

if ! [[ "${min_lean_formalized_args}" =~ ^[0-9]+$ ]]; then
  echo "Invalid value for --min-lean-formalized-args: ${min_lean_formalized_args}" >&2
  exit 1
fi
if ! [[ "${enforce_formal_track}" =~ ^[01]$ ]]; then
  echo "Invalid PF_FORMAL_TRACK_BLOCKING value: ${enforce_formal_track} (expected 0 or 1)" >&2
  exit 1
fi

if [[ ${#models[@]} -eq 0 ]]; then
  usage
  exit 1
fi

if [[ "${enforce_implementation_policy}" -eq 1 && -z "${implementation_policy_path}" ]]; then
  echo "--enforce-implementation-policy requires --implementation-policy <path>" >&2
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
  ddd_pim_file="${model_output_dir}/ddd-pim.md"
  sysml2_text_file="${model_output_dir}/sysml2.txt"
  sysml2_json_file="${model_output_dir}/sysml2.json"
  trace_map_json_file="${model_output_dir}/trace-map.json"
  alloy_file="${model_output_dir}/model.als"
  traceability_md_file="${model_output_dir}/traceability.md"
  traceability_csv_file="${model_output_dir}/traceability.csv"
  adequacy_differential_file="${model_output_dir}/adequacy-differential.md"
  adequacy_json_file="${model_output_dir}/adequacy-evidence.json"
  adequacy_status_file="${model_output_dir}/adequacy.status"
  implementation_trace_file="${model_output_dir}/implementation-trace.md"
  implementation_trace_status_file="${model_output_dir}/implementation-trace.status"
  implementation_trace_policy_status_file="${model_output_dir}/implementation-trace.policy.status"
  wrspm_file="${model_output_dir}/wrspm.md"
  wrspm_json_file="${model_output_dir}/wrspm.json"
  lean_model_file="${model_output_dir}/lean-model.lean"
  lean_check_json_file="${model_output_dir}/lean-check.json"
  lean_check_status_file="${model_output_dir}/lean-check.status"
  lean_differential_md_file="${model_output_dir}/lean-differential.md"
  lean_differential_json_file="${model_output_dir}/lean-differential.json"
  lean_differential_status_file="${model_output_dir}/lean-differential.status"
  formal_closure_report_file="${model_output_dir}/formal-closure.md"
  formal_closure_json_file="${model_output_dir}/formal-closure.json"
  formal_closure_status_file="${model_output_dir}/formal-closure.status"
  formal_closure_rows_tsv_file="${model_output_dir}/formal-closure.rows.tsv"
  formal_gap_report_file="${model_output_dir}/formal-gap.md"
  formal_gap_json_file="${model_output_dir}/formal-gap.json"
  formal_gap_status_file="${model_output_dir}/formal-gap.status"
  alloy_solver_dir="${model_output_dir}/alloy-solver"
  alloy_solver_report_file="${model_output_dir}/alloy-solver.md"
  alloy_solver_json_file="${model_output_dir}/alloy-solver.json"
  alloy_solver_status_file="${model_output_dir}/alloy-solver.status"
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
  cargo run -p pf_dsl -- "${model}" --ddd-pim > "${ddd_pim_file}"
  cargo run -p pf_dsl -- "${model}" --sysml2-text > "${sysml2_text_file}"
  cargo run -p pf_dsl -- "${model}" --sysml2-json > "${sysml2_json_file}"
  cargo run -p pf_dsl -- "${model}" --trace-map-json > "${trace_map_json_file}"
  cargo run -p pf_dsl -- "${model}" --alloy > "${alloy_file}"
  cargo run -p pf_dsl -- "${model}" --traceability-md "${traceability_args[@]}" > "${traceability_md_file}"
  cargo run -p pf_dsl -- "${model}" --traceability-csv "${traceability_args[@]}" > "${traceability_csv_file}"
  bash "${REPO_ROOT}/scripts/run_adequacy_evidence.sh" \
    --output "${adequacy_differential_file}" \
    --json "${adequacy_json_file}" \
    --status-file "${adequacy_status_file}"
  cargo run -p pf_dsl -- "${model}" --lean-model > "${lean_model_file}"
  bash "${REPO_ROOT}/scripts/run_lean_formal_check.sh" \
    --model "${model}" \
    --min-formalized-args "${min_lean_formalized_args}" \
    --output-dir "${model_output_dir}"
  bash "${REPO_ROOT}/scripts/run_lean_differential_check.sh" \
    --model "${model}" \
    --lean-status-json "${lean_check_json_file}" \
    --output "${lean_differential_md_file}" \
    --json "${lean_differential_json_file}" \
    --status-file "${lean_differential_status_file}" \
    --output-dir "${model_output_dir}"
  bash "${REPO_ROOT}/scripts/check_requirement_formal_closure.sh" \
    --model "${model}" \
    --lean-coverage-json "${model_output_dir}/lean-coverage.json" \
    --output "${formal_closure_report_file}" \
    --json "${formal_closure_json_file}" \
    --status-file "${formal_closure_status_file}" \
    --rows-tsv "${formal_closure_rows_tsv_file}"
  bash "${REPO_ROOT}/scripts/generate_formal_gap_report.sh" \
    --model "${model}" \
    --closure-rows-tsv "${formal_closure_rows_tsv_file}" \
    --traceability-csv "${traceability_csv_file}" \
    --output "${formal_gap_report_file}" \
    --json "${formal_gap_json_file}" \
    --status-file "${formal_gap_status_file}"
  bash "${REPO_ROOT}/scripts/run_alloy_solver_check.sh" \
    --model "${model}" \
    --alloy-file "${alloy_file}" \
    --output-dir "${alloy_solver_dir}" \
    --report "${alloy_solver_report_file}" \
    --json "${alloy_solver_json_file}" \
    --status-file "${alloy_solver_status_file}"
  trace_check_args=(
    --traceability-csv "${traceability_csv_file}"
    --output "${implementation_trace_file}"
    --status-file "${implementation_trace_status_file}"
    --policy-status-file "${implementation_trace_policy_status_file}"
  )
  if [[ -n "${implementation_policy_path}" ]]; then
    trace_check_args+=(--policy "${implementation_policy_path}")
  fi
  if [[ "${enforce_implementation_policy}" -eq 1 ]]; then
    trace_check_args+=(--enforce-policy)
  fi
  trace_check_args+=("${model}")
  bash "${REPO_ROOT}/scripts/check_model_implementation_trace.sh" "${trace_check_args[@]}"
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
  trace_map_coverage_status="$(
    grep -E '"status": "' "${trace_map_json_file}" \
      | head -n 1 \
      | sed -E 's/.*"status": "([^"]+)".*/\1/'
  )"
  trace_map_coverage_status="${trace_map_coverage_status:-UNKNOWN}"
  adequacy_status="$(cat "${adequacy_status_file}" 2>/dev/null || true)"
  adequacy_status="${adequacy_status:-UNKNOWN}"
  implementation_trace_status="$(cat "${implementation_trace_status_file}" 2>/dev/null || true)"
  implementation_trace_status="${implementation_trace_status:-UNKNOWN}"
  implementation_trace_policy_status="SKIPPED"
  if [[ -f "${implementation_trace_policy_status_file}" ]]; then
    implementation_trace_policy_status="$(cat "${implementation_trace_policy_status_file}" 2>/dev/null || true)"
    implementation_trace_policy_status="${implementation_trace_policy_status:-UNKNOWN}"
  fi
  lean_check_status="$(cat "${lean_check_status_file}" 2>/dev/null || true)"
  lean_check_status="${lean_check_status:-UNKNOWN}"
  lean_coverage_status="$(
    grep -E '"coverage_status": "' "${lean_check_json_file}" 2>/dev/null \
      | head -n 1 \
      | sed -E 's/.*"coverage_status": "([^"]+)".*/\1/' || true
  )"
  lean_coverage_status="${lean_coverage_status:-UNKNOWN}"
  lean_formalized_count="$(
    grep -E '"formalized_count": ' "${lean_check_json_file}" 2>/dev/null \
      | head -n 1 \
      | sed -E 's/.*"formalized_count": *([0-9]+).*/\1/' || true
  )"
  lean_formalized_count="${lean_formalized_count:-0}"
  lean_total_arguments="$(
    grep -E '"total_correctness_arguments": ' "${lean_check_json_file}" 2>/dev/null \
      | head -n 1 \
      | sed -E 's/.*"total_correctness_arguments": *([0-9]+).*/\1/' || true
  )"
  lean_total_arguments="${lean_total_arguments:-0}"
  lean_differential_status="$(cat "${lean_differential_status_file}" 2>/dev/null || true)"
  lean_differential_status="${lean_differential_status:-UNKNOWN}"
  formal_closure_status="$(cat "${formal_closure_status_file}" 2>/dev/null || true)"
  formal_closure_status="${formal_closure_status:-UNKNOWN}"
  formal_gap_status="$(cat "${formal_gap_status_file}" 2>/dev/null || true)"
  formal_gap_status="${formal_gap_status:-UNKNOWN}"
  alloy_solver_status="$(cat "${alloy_solver_status_file}" 2>/dev/null || true)"
  alloy_solver_status="${alloy_solver_status:-UNKNOWN}"
  formal_track_policy_mode="non_blocking"
  if [[ "${enforce_formal_track}" -eq 1 ]]; then
    formal_track_policy_mode="blocking"
  fi

  {
    echo "# PF Quality Gate Summary"
    echo
    echo "- Generated (UTC): \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`"
    echo "- Model: \`${model}\`"
    echo "- Decomposition closure status: \`${closure_status}\`"
    echo "- Concern coverage status: \`${concern_coverage_status}\`"
    echo "- Trace-map coverage status: \`${trace_map_coverage_status}\`"
    echo "- Adequacy evidence status: \`${adequacy_status}\`"
    echo "- Implementation trace status: \`${implementation_trace_status}\`"
    echo "- Implementation trace policy status: \`${implementation_trace_policy_status}\`"
    echo "- Lean formal check status: \`${lean_check_status}\`"
    echo "- Lean formal coverage status: \`${lean_coverage_status}\` (${lean_formalized_count}/${lean_total_arguments} formalized)"
    echo "- Lean differential status: \`${lean_differential_status}\`"
    echo "- Requirement formal closure status: \`${formal_closure_status}\`"
    echo "- Formal gap status: \`${formal_gap_status}\`"
    echo "- Alloy solver status: \`${alloy_solver_status}\`"
    echo "- Formal track policy mode: \`${formal_track_policy_mode}\`"
    echo
    echo "## Artifacts"
    echo
    echo "- \`report.md\`"
    echo "- \`decomposition-closure.md\`"
    echo "- \`obligations.md\`"
    echo "- \`concern-coverage.md\`"
    echo "- \`ddd-pim.md\`"
    echo "- \`sysml2.txt\`"
    echo "- \`sysml2.json\`"
    echo "- \`trace-map.json\`"
    echo "- \`model.als\`"
    echo "- \`traceability.md\`"
    echo "- \`traceability.csv\`"
    echo "- \`adequacy-differential.md\`"
    echo "- \`adequacy-evidence.json\`"
    echo "- \`implementation-trace.md\`"
    echo "- \`implementation-trace.policy.status\`"
    echo "- \`lean-model.lean\`"
    echo "- \`lean-coverage.json\`"
    echo "- \`lean-check.json\`"
    echo "- \`lean-differential.md\`"
    echo "- \`lean-differential.json\`"
    echo "- \`formal-closure.md\`"
    echo "- \`formal-closure.json\`"
    echo "- \`formal-closure.rows.tsv\`"
    echo "- \`formal-gap.md\`"
    echo "- \`formal-gap.json\`"
    echo "- \`alloy-solver.md\`"
    echo "- \`alloy-solver.json\`"
    echo "- \`alloy-solver/exec/receipt.json\`"
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
  if [[ "${trace_map_coverage_status}" != "PASS" ]]; then
    echo "Trace-map coverage failed for ${model}; generated targets are not fully mapped." >&2
    failure_count=$((failure_count + 1))
  fi
  if [[ "${implementation_trace_status}" != "PASS" && "${enforce_implementation_trace}" -eq 1 ]]; then
    echo "Implementation trace is ${implementation_trace_status} for ${model}; re-run without --enforce-implementation-trace to treat as non-blocking." >&2
    failure_count=$((failure_count + 1))
  fi
  if [[ "${implementation_trace_policy_status}" != "PASS" && "${enforce_implementation_policy}" -eq 1 ]]; then
    echo "Implementation trace policy is ${implementation_trace_policy_status} for ${model}; fix policy violations or relax policy thresholds." >&2
    failure_count=$((failure_count + 1))
  fi
  if [[ "${enforce_formal_track}" -eq 1 ]]; then
    if [[ "${lean_coverage_status}" != "PASS" ]]; then
      echo "Formal-track policy blocking: Lean coverage is ${lean_coverage_status} for ${model}." >&2
      failure_count=$((failure_count + 1))
    fi
    if [[ "${formal_closure_status}" != "PASS" ]]; then
      echo "Formal-track policy blocking: requirement formal closure is ${formal_closure_status} for ${model}." >&2
      failure_count=$((failure_count + 1))
    fi
    if [[ "${formal_gap_status}" != "PASS" ]]; then
      echo "Formal-track policy blocking: formal gap status is ${formal_gap_status} for ${model}." >&2
      failure_count=$((failure_count + 1))
    fi
    if [[ "${alloy_solver_status}" != "PASS" ]]; then
      echo "Formal-track policy blocking: Alloy solver status is ${alloy_solver_status} for ${model}." >&2
      failure_count=$((failure_count + 1))
    fi
  fi
done

if [[ "${failure_count}" -ne 0 ]]; then
  exit 1
fi
