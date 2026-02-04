#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
BIN=(cargo run -p bijux_dag_cli --)
TMP=$(mktemp -d)
RUNS="$TMP/runs"
mkdir -p "$RUNS"

"${BIN[@]}" run "$ROOT/examples/hello.dag.json" --out "$RUNS" >/dev/null
"${BIN[@]}" run "$ROOT/examples/hello.dag.json" --out "$RUNS" >/dev/null

RUN2=$(ls -dt "$RUNS"/run-* | head -n 1)
RUN1=$(ls -dt "$RUNS"/run-* | head -n 2 | tail -n 1)

"${BIN[@]}" diff "$RUN1" "$RUN2" --json > "$TMP/diff.json"
python3 - "$TMP/diff.json" <<'PY'
import json,sys
with open(sys.argv[1]) as f:
    d=json.load(f)
assert d.get("manifest") == {}, d
assert d.get("graph_fingerprint") is None, d
assert d.get("nodes") == {}, d
assert d.get("outputs") == {}, d
PY

"${BIN[@]}" replay "$RUN2" --out "$RUNS" >/dev/null
REPLAY=$(ls -dt "$RUNS"/run-* | head -n 1)
"${BIN[@]}" diff "$RUN2" "$REPLAY" --json > "$TMP/diff_replay.json"
python3 - "$TMP/diff_replay.json" <<'PY'
import json,sys
with open(sys.argv[1]) as f:
    d=json.load(f)
assert d.get("manifest") == {}, d
assert d.get("graph_fingerprint") is None, d
assert d.get("nodes") == {}, d
assert d.get("outputs") == {}, d
PY
