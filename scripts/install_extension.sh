#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
EXT_DIR="${HOME}/.vscode/extensions/pf-dsl.problem-frames"

echo "Building LSP..."
cargo build --manifest-path "${REPO_ROOT}/crates/pf_lsp/Cargo.toml" --release
cp "${REPO_ROOT}/target/release/pf_lsp" "${REPO_ROOT}/editors/code/pf_lsp"

echo "Building extension..."
pushd "${REPO_ROOT}/editors/code" >/dev/null
npm ci
npm run compile
popd >/dev/null

echo "Installing extension files to ${EXT_DIR}..."
rm -rf "${EXT_DIR}"
mkdir -p "${EXT_DIR}"
cp -R "${REPO_ROOT}/editors/code/." "${EXT_DIR}"

echo "Installed. Restart VS Code or run: Developer: Reload Window"
