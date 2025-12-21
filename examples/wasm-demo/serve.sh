#!/bin/bash
# Simple HTTP server for the WASM demo
# Runs on localhost:3000

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if pkg directory exists
if [ ! -d "pkg" ]; then
    echo "Error: pkg directory not found!"
    echo "Run ./build-wasm.sh from the project root first, then run this script."
    exit 1
fi

# Check if deno is available
if ! command -v deno &> /dev/null; then
    echo "Error: Deno not found. Please install Deno: https://deno.land/"
    exit 1
fi

echo "Starting server at http://localhost:3000"
echo "Press Ctrl+C to stop"
echo ""

deno run --allow-net --allow-read jsr:@std/http/file-server --port 3000
