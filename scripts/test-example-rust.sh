#!/usr/bin/env bash
set -euo pipefail

just build-example-rust

OUTPUT="$(just run-example-rust)"
printf '%s\n' "$OUTPUT"

if grep -q "error" <<<"$OUTPUT"; then
  echo "1 or more error"
  exit 1
fi

echo "rust example tests pass"
