sudo := "$(docker info > /dev/null 2>&1 || echo 'sudo')"

export LIBSQLITE3_FLAGS := "-DSQLITE_THREADSAFE=0"

build-sqlite:
  export $(cat .env | xargs) && cargo build -p sqlite-component --release

plug-rust:
  wac plug \
    target/wasm32-wasip2/release/example-rust.wasm \
    --plug target/wasm32-wasip2/release/sqlite_component.wasm \
    -o target/wasm32-wasip2/release/example-rust-total.wasm

build-rust:
  just build-sqlite
  cargo build --manifest-path example-rust/Cargo.toml --release
  just plug-rust

plug-js:
  wac plug \
    target/wasm32-wasip2/release/example-js.wasm \
    --plug target/wasm32-wasip2/release/sqlite_component.wasm \
    -o target/wasm32-wasip2/release/example-js-total.wasm

build-js:
  just build-sqlite
  npm --prefix example-js install
  npm --prefix example-js run build
  just plug-js

build:
  just build-rust
  just build-js

run-rust:
  mkdir -p mount/
  wasmtime run --dir ./mount::/workspace target/wasm32-wasip2/release/example-rust-total.wasm

run-js:
  mkdir -p mount/
  wasmtime run --dir ./mount::/workspace target/wasm32-wasip2/release/example-js-total.wasm

build-docker:
  {{sudo}} docker build -f Dockerfile -t sqlitewasi .
