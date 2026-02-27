FROM rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
  wget \
  build-essential \
  clang

RUN mkdir -p /root/sdk
WORKDIR /root/sdk

COPY scripts/wasi-sdk.sh .
RUN chmod +x wasi-sdk.sh
RUN ./wasi-sdk.sh

WORKDIR /workspace
RUN rustup target add wasm32-wasip2
COPY Cargo.toml .
COPY Cargo.lock .
COPY .cargo/ /workspace/.cargo/
COPY component/ /workspace/component/
COPY example-rust/ /workspace/example-rust/
COPY wit/ /workspace/wit/
RUN WASI_SDK_PATH="/root/wasi-sdk-27.0-x86_64-linux" LIBSQLITE3_FLAGS="-DSQLITE_THREADSAFE=0" cargo build -p component --release

FROM scratch AS export
COPY --from=builder /workspace/target/wasm32-wasip2/release/component.wasm /component.wasm
