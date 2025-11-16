#!/bin/bash
# Build script for WebAssembly package
set -e

echo "Building fpexif for WebAssembly..."

# Check if wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build the library with wasm feature for wasm32-unknown-unknown target
echo "Compiling to WASM..."
cargo build --lib --target wasm32-unknown-unknown --release --features wasm

# Generate JavaScript bindings using wasm-bindgen
echo "Generating JavaScript bindings..."
wasm-bindgen \
    --target web \
    --out-dir pkg \
    target/wasm32-unknown-unknown/release/fpexif.wasm

echo "✓ WASM build complete! Output in ./pkg/"
echo "  - fpexif_bg.wasm (WebAssembly binary)"
echo "  - fpexif.js (JavaScript bindings)"
echo "  - fpexif.d.ts (TypeScript definitions)"
