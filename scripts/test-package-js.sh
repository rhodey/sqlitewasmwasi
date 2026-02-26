#!/usr/bin/env bash
set -euo pipefail

just build-js

OUTPUT="$(just run-js)"
printf '%s\n' "$OUTPUT"

if grep -q "fail" <<<"$OUTPUT"; then
  echo "1 or more fail"
  exit 1
fi

echo "tests pass"
