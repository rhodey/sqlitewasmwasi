sudo := "$(docker info > /dev/null 2>&1 || echo 'sudo')"

export LIBSQLITE3_FLAGS := "-DSQLITE_THREADSAFE=0"

component:
  export $(cat .env | xargs) && cargo build -p component --release

component-docker:
  mkdir -p target/wasm32-wasip2/release
  {{sudo}} docker build -f Dockerfile -t component --target export .
  {{sudo}} docker build --output type=local,dest=./target/wasm32-wasip2/release --target export .

plug-example-rust:
  wac plug \
    target/wasm32-wasip2/release/example-rust.wasm \
    --plug target/wasm32-wasip2/release/component.wasm \
    -o target/wasm32-wasip2/release/example-rust-total.wasm

build-example-rust:
  cargo build --manifest-path example-rust/Cargo.toml --release
  just plug-example-rust

run-example-rust:
  mkdir -p app/
  wasmtime run --dir ./app::/app target/wasm32-wasip2/release/example-rust-total.wasm

plug-package-js:
  wac plug \
    package-js/dist/test.js.wasm \
    --plug target/wasm32-wasip2/release/component.wasm \
    -o target/wasm32-wasip2/release/test.js.total.wasm

build-package-js:
  npm --prefix package-js install
  npm --prefix package-js run build
  just plug-package-js

run-package-js:
  mkdir -p app/
  wasmtime run --dir ./app::/app target/wasm32-wasip2/release/test.js.total.wasm

plug-example-js:
  wac plug \
    example-js/dist/example.js.wasm \
    --plug target/wasm32-wasip2/release/component.wasm \
    -o target/wasm32-wasip2/release/example.js.total.wasm

build-example-js:
  npm --prefix example-js install
  npm --prefix example-js run build
  just plug-example-js

run-example-js:
  mkdir -p app/
  wasmtime run --dir ./app::/app target/wasm32-wasip2/release/example.js.total.wasm

build:
  just component
  just build-example-rust
  just build-package-js

test:
  just build
  bash -c scripts/test-example-rust.sh
  bash -c scripts/test-package-js.sh
