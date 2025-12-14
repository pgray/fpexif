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
cargo build --lib --target wasm32-unknown-unknown --features wasm

# Generate JavaScript bindings using wasm-bindgen
echo "Generating JavaScript bindings..."
wasm-bindgen \
    --target web \
    --out-dir pkg \
    target/wasm32-unknown-unknown/debug/fpexif.wasm

echo "✓ WASM build complete! Output in ./pkg/"
echo "  - fpexif_bg.wasm (WebAssembly binary)"
echo "  - fpexif.js (JavaScript bindings)"
echo "  - fpexif.d.ts (TypeScript definitions)"

# Copy to examples/wasm-demo for the demo server
echo ""
echo "Copying to examples/wasm-demo..."
rm -rf examples/wasm-demo/pkg
cp -r pkg examples/wasm-demo/

echo ""
echo "To run the demo:"
echo "  cd examples/wasm-demo && ./serve.sh"
echo "  Then open http://localhost:3000"
