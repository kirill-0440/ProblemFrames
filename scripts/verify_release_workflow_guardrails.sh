#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

WORKFLOW_PATH="${REPO_ROOT}/.github/workflows/release-artifacts.yml"
SYFT_CONFIG_PATH="${REPO_ROOT}/.github/syft-release.yaml"

if [[ ! -f "${WORKFLOW_PATH}" ]]; then
  echo "Missing workflow file: ${WORKFLOW_PATH}" >&2
  exit 1
fi

if [[ ! -f "${SYFT_CONFIG_PATH}" ]]; then
  echo "Missing Syft config file: ${SYFT_CONFIG_PATH}" >&2
  exit 1
fi

# Parse YAML with stdlib parser to fail fast on syntax errors.
ruby -e 'require "yaml"; YAML.load_file(ARGV[0]); YAML.load_file(ARGV[1])' \
  "${WORKFLOW_PATH}" \
  "${SYFT_CONFIG_PATH}"

publish_release_block="$(
  awk '
    /^  publish-release:/ { in_block=1 }
    in_block && /^  [A-Za-z0-9_-]+:/ && $0 !~ /^  publish-release:/ { in_block=0 }
    in_block { print }
  ' "${WORKFLOW_PATH}"
)"

if [[ -z "${publish_release_block}" ]]; then
  echo "publish-release job block not found in ${WORKFLOW_PATH}" >&2
  exit 1
fi

require_in_publish_release() {
  local pattern="$1"
  local description="$2"
  if ! grep -qE "${pattern}" <<<"${publish_release_block}"; then
    echo "Missing release guardrail in publish-release: ${description}" >&2
    exit 1
  fi
}

require_in_publish_release 'name: Checkout' 'repository checkout step'
require_in_publish_release 'name: Generate release SBOM' 'SBOM generation step'
require_in_publish_release 'config: \.github/syft-release\.yaml' 'Syft config wiring'
require_in_publish_release 'syft-version: v1\.38\.0' 'pinned Syft version'
require_in_publish_release 'name: Validate SBOM checksums for release binaries' 'SBOM checksum validation step'

if ! grep -qE 'selection:[[:space:]]*all' "${SYFT_CONFIG_PATH}"; then
  echo "Syft config must keep file.metadata.selection=all for release fidelity" >&2
  exit 1
fi

if ! grep -qE -- '-[[:space:]]*sha256' "${SYFT_CONFIG_PATH}"; then
  echo "Syft config must include sha256 digest collection" >&2
  exit 1
fi

echo "Release workflow guardrails are configured."
