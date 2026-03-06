#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/artifacts/benchmarks"
RUNS_DIR="${OUT_DIR}/runs"
REPORT="${OUT_DIR}/baseline.json"

mkdir -p "${RUNS_DIR}"
START_MS=$(date +%s%3N)

cargo run -p bijux-dag-cli -- dag run "${ROOT_DIR}/benchmarks/fixtures/large_dag.json" --out "${RUNS_DIR}" >/dev/null

END_MS=$(date +%s%3N)
ELAPSED_MS=$((END_MS - START_MS))

cat > "${REPORT}" <<JSON
{
  "fixture": "benchmarks/fixtures/large_dag.json",
  "elapsed_ms": ${ELAPSED_MS},
  "recorded_at_unix_ms": ${END_MS}
}
JSON

echo "wrote ${REPORT}"
