#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/artifacts/memory"
RUNS_DIR="${OUT_DIR}/runs"
REPORT="${OUT_DIR}/smoke.json"

mkdir -p "${RUNS_DIR}"
START_MS=$(date +%s%3N)

cargo run -p bijux-dag-cli -- dag run "${ROOT_DIR}/examples/hello.dag.json" --out "${RUNS_DIR}" >/dev/null

END_MS=$(date +%s%3N)
ELAPSED_MS=$((END_MS - START_MS))

cat > "${REPORT}" <<JSON
{
  "workload": "examples/hello.dag.json",
  "elapsed_ms": ${ELAPSED_MS},
  "memory_budget_note": "Track peak memory through CI runner metrics and fail on sustained regressions.",
  "recorded_at_unix_ms": ${END_MS}
}
JSON

echo "wrote ${REPORT}"
