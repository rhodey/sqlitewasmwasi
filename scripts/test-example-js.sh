#!/usr/bin/env bash
set -euo pipefail

just build-example-js

OUTPUT="$(just run-example-js)"
printf '%s\n' "$OUTPUT"

if grep -q "error" <<<"$OUTPUT"; then
  echo "1 or more error"
  exit 1
fi

echo "js example tests pass"
