#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v wac >/dev/null 2>&1; then
  echo "error: wac is required (cargo install wac-cli)" >&2
  exit 1
fi

WASMTIME_BIN="${WASMTIME_BIN:-$HOME/.wasmtime/bin/wasmtime}"
if [[ ! -x "$WASMTIME_BIN" ]]; then
  if command -v wasmtime >/dev/null 2>&1; then
    WASMTIME_BIN="$(command -v wasmtime)"
  else
    echo "error: wasmtime is required (https://wasmtime.dev/)" >&2
    exit 1
  fi
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "error: npm is required to build the ComponentizeJS example" >&2
  exit 1
fi

if [[ -z "${WASI_SDK_PATH:-}" && -d /opt/wasi-sdk-25 ]]; then
  export WASI_SDK_PATH=/opt/wasi-sdk-25
fi

if [[ -n "${WASI_SDK_PATH:-}" ]]; then
  export CC_wasm32_wasip2="${CC_wasm32_wasip2:-$WASI_SDK_PATH/bin/clang}"
fi

# WASI P2 libc in this project builds sqlite without thread support.
export LIBSQLITE3_FLAGS="${LIBSQLITE3_FLAGS:- -DSQLITE_THREADSAFE=0}"

cargo build -p sqlite-component --target wasm32-wasip2 --release

mkdir -p target/wasm32-wasip2/release
cp target/wasm32-wasip2/release/sqlite_component.wasm target/wasm32-wasip2/release/sqlite-component.wasm

npm --prefix examples/js-client install
npm --prefix examples/js-client run build

wac plug \
  target/wasm32-wasip2/release/js-client.wasm \
  --plug target/wasm32-wasip2/release/sqlite-component.wasm \
  -o target/wasm32-wasip2/release/js-client-composed.wasm

WASI_DB_DIR="${WASI_DB_DIR:-$ROOT_DIR/target/wasi-db-js-client}"
mkdir -p "$WASI_DB_DIR"

OUTPUT="$($WASMTIME_BIN run --dir "$WASI_DB_DIR"::/workspace target/wasm32-wasip2/release/js-client-composed.wasm)"
printf '%s\n' "$OUTPUT"

grep -q '^id=int=1$' <<<"$OUTPUT"
grep -q '^name=text=hello from rust$' <<<"$OUTPUT"
grep -q '^note=null$' <<<"$OUTPUT"
grep -q '^ratio=real=3.25$' <<<"$OUTPUT"
grep -q '^big_id=int=9007199254740993$' <<<"$OUTPUT"
grep -q '^one() got single row back$' <<<"$OUTPUT"

echo "wasmtime output validation passed"
