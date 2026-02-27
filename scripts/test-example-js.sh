#!/usr/bin/env bash
set -euo pipefail

just build-example-js

just run-example-js
