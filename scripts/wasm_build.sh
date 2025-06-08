#!/usr/bin/env bash
# Build WebAssembly package for lmdb-tui using wasm-pack.
# Requires https://rustwasm.github.io/wasm-pack/ to be installed.
set -euo pipefail

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found. Install with: cargo install wasm-pack" >&2
  exit 1
fi

wasm-pack build --target web --release

