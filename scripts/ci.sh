#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

make fmt
make lint
./scripts/dep_guard.sh
make test
make golden
cargo run -p bijux_cli -- dag compat >/dev/null

TMP=$(mktemp -d)
RUNS="$TMP/runs"
mkdir -p "$RUNS"

cargo run -p bijux_cli -- dag run "$ROOT/examples/hello.dag.json" --out "$RUNS" >/dev/null
RUN=$(ls -dt "$RUNS"/run-* | head -n 1)

cargo run -p bijux_cli -- dag verify "$RUN" >/dev/null
