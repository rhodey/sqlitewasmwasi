#!/usr/bin/env bash
set -e

WASI_OS=linux
WASI_ARCH=x86_64
WASI_VERSION=27
WASI_VERSION_FULL="${WASI_VERSION}.0"
url="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}-${WASI_ARCH}-${WASI_OS}.tar.gz"
wget "$url" -O sdk.tar.gz
tar xvf sdk.tar.gz -C ../
