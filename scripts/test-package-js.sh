#!/usr/bin/env bash
set -euo pipefail

just build-package-js

OUTPUT="$(just run-package-js)"
printf '%s\n' "$OUTPUT"

if grep -q "fail" <<<"$OUTPUT"; then
  echo "1 or more fail"
  exit 1
fi

if grep -q "error" <<<"$OUTPUT"; then
  echo "1 or more error"
  exit 1
fi

echo "js tests pass"
