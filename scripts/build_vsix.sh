#!/bin/bash
set -e

# Ensure vsce is installed (or use npx)
if ! command -v vsce &> /dev/null; then
    echo "Installing vsce..."
    npm install -g @vscode/vsce
fi

echo "Building LSP binary..."
cd crates/pf_lsp
cargo build --release
# Copy binary to extension folder
cp target/release/pf_lsp ../../editors/code/pf_lsp

echo "Packaging VSIX..."
cd ../../editors/code
npm install
npm run compile
npx vsce package

echo "VSIX created at editors/code/problem-frames-0.0.1.vsix"
