#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$repo_root"

echo "[e2e] running crate-level integration entrypoint"
cargo test -p bijux-dag-app --test e2e_integration_scenarios

echo "[e2e] running local binary entrypoint smoke"
cargo run -p bijux-dag-cli -- dag validate examples/hello.dag.json >/dev/null

echo "[e2e] matrix completed"
