FROM rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
  wget \
  build-essential \
  clang

WORKDIR /root
COPY <<EOF /root/sdk.sh
WASI_OS=linux
WASI_ARCH=x86_64
WASI_VERSION=27
WASI_VERSION_FULL="\${WASI_VERSION}.0"
url="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-\${WASI_VERSION}/wasi-sdk-\${WASI_VERSION_FULL}-\${WASI_ARCH}-\${WASI_OS}.tar.gz"
wget "\$url" -O sdk.tar.gz
mkdir -p /root/sdk
tar xvf sdk.tar.gz -C /root/sdk
EOF
RUN chmod +x /root/sdk.sh
RUN /root/sdk.sh

WORKDIR /workspace
RUN rustup target add wasm32-wasip2
COPY Cargo.toml .
COPY Cargo.lock .
COPY .cargo/ /workspace/.cargo/
COPY component/ /workspace/component/
COPY example-rust/ /workspace/example-rust/
COPY wit/ /workspace/wit/
RUN WASI_SDK_PATH="/root/sdk/wasi-sdk-27.0-x86_64-linux" LIBSQLITE3_FLAGS="-DSQLITE_THREADSAFE=0" cargo build -p component --release

FROM scratch AS export
COPY --from=builder /workspace/target/wasm32-wasip2/release/component.wasm /component.wasm
