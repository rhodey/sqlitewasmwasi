#!/usr/bin/env bash
set -euo pipefail

just build-package-rust

OUTPUT="$(just run-package-rust)"
printf '%s\n' "$OUTPUT"

if grep -q "error" <<<"$OUTPUT"; then
  echo "1 or more error"
  exit 1
fi

echo "rust package tests pass"
