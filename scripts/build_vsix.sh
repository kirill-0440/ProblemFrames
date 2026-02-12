#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

echo "Building LSP binary..."
cargo build --manifest-path "${REPO_ROOT}/crates/pf_lsp/Cargo.toml" --release
cp "${REPO_ROOT}/target/release/pf_lsp" "${REPO_ROOT}/editors/code/pf_lsp"

echo "Packaging VSIX..."
pushd "${REPO_ROOT}/editors/code" >/dev/null
npm ci
npm run compile
npx -y @vscode/vsce package
popd >/dev/null

echo "VSIX created at ${REPO_ROOT}/editors/code/problem-frames-0.0.1.vsix"
