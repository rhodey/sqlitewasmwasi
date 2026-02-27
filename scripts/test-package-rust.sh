#!/usr/bin/env bash
set -euo pipefail

cargo build --manifest-path package-rust/Cargo.toml --bin test-rust --release
mkdir -p package-rust/dist
cp target/wasm32-wasip2/release/test-rust.wasm package-rust/dist/test.rust.wasm

wac plug \
  package-rust/dist/test.rust.wasm \
  --plug target/wasm32-wasip2/release/component.wasm \
  -o target/wasm32-wasip2/release/test.rust.total.wasm

mkdir -p app/
OUTPUT="$(wasmtime run --dir ./app::/app target/wasm32-wasip2/release/test.rust.total.wasm)"
printf '%s\n' "$OUTPUT"

if grep -q "error" <<<"$OUTPUT"; then
  echo "1 or more error"
  exit 1
fi

echo "rust package tests pass"
