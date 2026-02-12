#!/bin/bash
set -e

# npx will handle vsce execution

echo "Building LSP binary..."
cd crates/pf_lsp
cargo build --release
# Copy binary to extension folder
cp target/release/pf_lsp ../../editors/code/pf_lsp

echo "Packaging VSIX..."
cd ../../editors/code
npm install
npm run compile
npx -y @vscode/vsce package

echo "VSIX created at editors/code/problem-frames-0.0.1.vsix"
