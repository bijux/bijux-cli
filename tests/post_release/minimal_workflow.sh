#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

BIN="${BIJUX_RELEASE_BINARY:-$ROOT/target/debug/bijux}"

"$BIN" dag init --dir "$TMP_DIR" >/dev/null
"$BIN" dag validate "$TMP_DIR/dag.json" >/dev/null
"$BIN" dag run "$TMP_DIR/dag.json" --runs-dir "$TMP_DIR/runs" >/dev/null || true
"$BIN" dag status "$TMP_DIR/runs" >/dev/null || true

echo "post-release minimal workflow completed"
