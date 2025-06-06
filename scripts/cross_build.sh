#!/usr/bin/env bash
# Build release binaries for multiple targets using cross.
# Requires https://github.com/cross-rs/cross to be installed.
set -euo pipefail

# List of targets to build
TARGETS=(
  x86_64-unknown-linux-musl
  aarch64-unknown-linux-musl
  x86_64-apple-darwin
  aarch64-apple-darwin
  x86_64-pc-windows-gnu
  aarch64-pc-windows-gnu
)

# Determine package version from Cargo.toml
VERSION=$(grep '^version =' Cargo.toml | head -n1 | cut -d '"' -f2)

for target in "${TARGETS[@]}"; do
  echo "Building $target"
  cross build --release --target "$target"
  dir="target/$target/release"
  bin="lmdb-tui"
  [[ "$target" == *windows* ]] && bin+=".exe"
  archive="lmdb-tui-${VERSION}-${target}.tar.gz"
  (cd "$dir" && tar -czf "../../../${archive}" "$bin")
  echo "Created $archive"

done

mkdir -p dist
mv lmdb-tui-*.tar.gz dist/

