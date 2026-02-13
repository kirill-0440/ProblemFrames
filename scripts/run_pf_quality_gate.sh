#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

OUTPUT_DIR="${REPO_ROOT}/.ci-artifacts/pf-quality-gate"
IMPACT_SEEDS=""
IMPACT_HOPS=2
ALLOW_OPEN_CLOSURE=0
MODELS=()

usage() {
  cat <<'USAGE'
Usage:
  bash ./scripts/run_pf_quality_gate.sh [options] <model.pf> [more models...]

Options:
  --out-dir <path>         Output directory for generated artifacts.
  --impact <seeds>         Traceability impact seeds (example: requirement:R1,domain:D1).
  --impact-hops <n>        Traceability impact traversal depth (default: 2).
  --allow-open-closure     Do not fail on uncovered/orphan/boundary items.
  -h, --help               Show this help message.
USAGE
}

section_has_open_items() {
  local file="$1"
  local section="$2"

  awk -v section="$section" '
    $0 == "### " section {in_section = 1; next}
    in_section && /^### / {in_section = 0}
    in_section && /^- / && $0 != "- None." {open = 1}
    END {exit open ? 0 : 1}
  ' "$file"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      [[ $# -ge 2 ]] || { echo "Missing value for --out-dir" >&2; exit 1; }
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --impact)
      [[ $# -ge 2 ]] || { echo "Missing value for --impact" >&2; exit 1; }
      IMPACT_SEEDS="$2"
      shift 2
      ;;
    --impact-hops)
      [[ $# -ge 2 ]] || { echo "Missing value for --impact-hops" >&2; exit 1; }
      IMPACT_HOPS="$2"
      shift 2
      ;;
    --allow-open-closure)
      ALLOW_OPEN_CLOSURE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
    *)
      MODELS+=("$1")
      shift
      ;;
  esac
done

if [[ ${#MODELS[@]} -eq 0 ]]; then
  usage
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

failures=()

for model in "${MODELS[@]}"; do
  if [[ ! -f "${model}" ]]; then
    echo "Model file not found: ${model}" >&2
    failures+=("${model}")
    continue
  fi

  model_abs="$(cd -- "$(dirname -- "${model}")" && pwd)/$(basename -- "${model}")"
  if [[ "${model_abs}" == "${REPO_ROOT}/"* ]]; then
    model_rel="${model_abs#${REPO_ROOT}/}"
  else
    model_rel="$(basename -- "${model_abs}")"
  fi

  model_stem="${model_rel%.pf}"
  model_out="${OUTPUT_DIR}/${model_stem}"
  mkdir -p "${model_out}"

  echo "Running PF quality gate for ${model_rel}"

  cargo run -p pf_dsl -- "${model_abs}" --report > "${model_out}/report.md"
  cargo run -p pf_dsl -- "${model_abs}" --decomposition-closure > "${model_out}/decomposition-closure.md"
  cargo run -p pf_dsl -- "${model_abs}" --obligations > "${model_out}/obligations.md"
  cargo run -p pf_dsl -- "${model_abs}" --alloy > "${model_out}/model.als"

  traceability_args=()
  if [[ -n "${IMPACT_SEEDS}" ]]; then
    traceability_args+=("--impact=${IMPACT_SEEDS}")
    traceability_args+=("--impact-hops=${IMPACT_HOPS}")
  fi

  cargo run -p pf_dsl -- "${model_abs}" --traceability-md "${traceability_args[@]}" > "${model_out}/traceability.md"
  cargo run -p pf_dsl -- "${model_abs}" --traceability-csv "${traceability_args[@]}" > "${model_out}/traceability.csv"

  if [[ ${ALLOW_OPEN_CLOSURE} -eq 0 ]]; then
    closure_file="${model_out}/decomposition-closure.md"
    if section_has_open_items "${closure_file}" "Uncovered Requirements" \
      || section_has_open_items "${closure_file}" "Orphan Subproblems" \
      || section_has_open_items "${closure_file}" "Boundary Mismatches"; then
      echo "Decomposition closure is open for ${model_rel}. Review ${closure_file}." >&2
      failures+=("${model_rel}")
    fi
  fi

  echo "Artifacts written to ${model_out}"
done

if [[ ${#failures[@]} -gt 0 ]]; then
  echo "PF quality gate failed for: ${failures[*]}" >&2
  exit 1
fi

echo "PF quality gate passed for ${#MODELS[@]} model(s)."
