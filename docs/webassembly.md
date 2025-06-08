# WebAssembly Build

This project can be compiled to WebAssembly for running in the browser.
The build uses [`wasm-pack`](https://rustwasm.github.io/wasm-pack/).

## Prerequisites

Install **wasm-pack** if it is not already available:
```bash
cargo install wasm-pack
```

## Building

Run the helper script to produce a `pkg/` directory with the generated
`lmdb_tui.js` and `lmdb_tui_bg.wasm` files:
```bash
scripts/wasm_build.sh
```

The resulting files can be served by any static web server.
