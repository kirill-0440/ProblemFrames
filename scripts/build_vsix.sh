#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

if [[ "${PF_DRY_RUN:-0}" == "1" ]]; then
  echo "[dry-run] cargo build --manifest-path \"${REPO_ROOT}/crates/pf_lsp/Cargo.toml\" --release"
  echo "[dry-run] cp \"${REPO_ROOT}/target/release/pf_lsp*\" \"${REPO_ROOT}/editors/code/\""
  echo "[dry-run] cd \"${REPO_ROOT}/editors/code\""
  echo "[dry-run] npm ci"
  echo "[dry-run] npm run compile"
  echo "[dry-run] npx -y @vscode/vsce package"
  exit 0
fi

echo "Building LSP binary..."
cargo build --manifest-path "${REPO_ROOT}/crates/pf_lsp/Cargo.toml" --release
if [[ -f "${REPO_ROOT}/target/release/pf_lsp.exe" ]]; then
  BIN_NAME="pf_lsp.exe"
else
  BIN_NAME="pf_lsp"
fi
rm -f "${REPO_ROOT}/editors/code/pf_lsp" "${REPO_ROOT}/editors/code/pf_lsp.exe"
cp "${REPO_ROOT}/target/release/${BIN_NAME}" "${REPO_ROOT}/editors/code/${BIN_NAME}"

echo "Packaging VSIX..."
pushd "${REPO_ROOT}/editors/code" >/dev/null
npm ci
npm run compile
npx -y @vscode/vsce package
popd >/dev/null

echo "VSIX created at ${REPO_ROOT}/editors/code/problem-frames-0.0.1.vsix"
