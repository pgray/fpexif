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

echo "Starting server at http://localhost:3000"
echo "Press Ctrl+C to stop"
echo ""

# Try python3 first, then python
if command -v python3 &> /dev/null; then
    python3 -m http.server 3000
elif command -v python &> /dev/null; then
    python -m http.server 3000
else
    echo "Error: Python not found. Please install Python or use another HTTP server."
    exit 1
fi
