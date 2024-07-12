#!/usr/bin/env bash
set -euo pipefail

trap 'kill 0' EXIT

python3 -m http.server &
cargo watchreload
