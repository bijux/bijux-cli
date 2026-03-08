#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
cargo run -p bijux-dev-dag --bin bijux-dev-dag -- repo schema-changelog "$@"
