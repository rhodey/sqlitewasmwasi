#!/usr/bin/env bash
set -euo pipefail

just build-example-rust

OUTPUT="$(just run-example-rust)"
printf '%s\n' "$OUTPUT"

grep -q '^id=int=1$' <<<"$OUTPUT"
grep -q '^name=text=hello from rust$' <<<"$OUTPUT"
grep -q '^note=null$' <<<"$OUTPUT"
grep -q '^ratio=real=3.25$' <<<"$OUTPUT"
grep -q '^big_id=int=9007199254740993$' <<<"$OUTPUT"
grep -q '^one() got single row back$' <<<"$OUTPUT"

echo "rust tests pass"
