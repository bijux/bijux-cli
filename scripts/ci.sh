#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

make fmt
make lint
make test
make golden
cargo run -p bijux_dag_cli -- compat >/dev/null

TMP=$(mktemp -d)
RUNS="$TMP/runs"
mkdir -p "$RUNS"

cargo run -p bijux_dag_cli -- run "$ROOT/examples/hello.dag.json" --out "$RUNS" >/dev/null
RUN=$(ls -dt "$RUNS"/run-* | head -n 1)

cargo run -p bijux_dag_cli -- verify-run "$RUN" >/dev/null
