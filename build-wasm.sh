#!/bin/bash

echo "Building for WASM..."

# Install wasm-pack if not already installed  
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build for web
wasm-pack build --target web --out-dir pkg

echo "WASM build complete! You can now serve the project."
echo "Try: python3 -m http.server 8000"
echo "Then open: http://localhost:8000"
