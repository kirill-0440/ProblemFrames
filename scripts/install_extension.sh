#!/bin/bash
set -e

# Build the LSP
echo "Building LSP..."
cd ../crates/pf_lsp
cargo build --release
cp target/release/pf_lsp ../../editors/code/pf_lsp

# Build the Extension
echo "Building Extension..."
cd ../../editors/code
npm install
npm run compile

# Package (optional, but good practice)
# vsce package

echo "To install the extension:"
echo "1. Open VS Code"
echo "2. Go to Extensions -> ... -> Install from VSIX (if packaged)"
echo "OR for development:"
echo "1. Open 'editors/code' in VS Code"
echo "2. Press F5 to launch Validation/Debug"

echo "Since we didn't package it, you can simplify by symlinking or copying to ~/.vscode/extensions"
mkdir -p ~/.vscode/extensions/pf-dsl
cp -r ./* ~/.vscode/extensions/pf-dsl/
echo "installed to ~/.vscode/extensions/pf-dsl"
