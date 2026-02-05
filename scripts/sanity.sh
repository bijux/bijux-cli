#!/usr/bin/env bash
set -euo pipefail

cargo metadata --no-deps >/dev/null
cargo test -q
cargo fmt --all -- --check
