sudo := "$(docker info > /dev/null 2>&1 || echo 'sudo')"

export LIBSQLITE3_FLAGS := "-DSQLITE_THREADSAFE=0"

component:
  export $(cat .env | xargs) && cargo build -p component --release

component-docker:
  mkdir -p target/wasm32-wasip2/release
  {{sudo}} docker build -f Dockerfile -t component --target export .
  {{sudo}} docker build --output type=local,dest=./target/wasm32-wasip2/release --target export .

plug-rust:
  wac plug \
    target/wasm32-wasip2/release/example-rust.wasm \
    --plug target/wasm32-wasip2/release/component.wasm \
    -o target/wasm32-wasip2/release/example-rust-total.wasm

build-rust:
  cargo build --manifest-path example-rust/Cargo.toml --release
  just plug-rust

run-rust:
  mkdir -p app/
  wasmtime run --dir ./app::/app target/wasm32-wasip2/release/example-rust-total.wasm

plug-js:
  wac plug \
    package-js/dist/test.js.wasm \
    --plug target/wasm32-wasip2/release/component.wasm \
    -o target/wasm32-wasip2/release/test.js.total.wasm

build-js:
  npm --prefix package-js install
  npm --prefix package-js run build
  just plug-js

run-js:
  mkdir -p app/
  wasmtime run --dir ./app::/app target/wasm32-wasip2/release/test.js.total.wasm

build:
  just component
  just build-rust
  just build-js

run:
  just run-rust
  just run-js

test:
  just build
  bash -c scripts/test-example-rust.sh
  bash -c scripts/test-package-js.sh
