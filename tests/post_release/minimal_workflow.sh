#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

"$ROOT/target/debug/bijux" dag init --dir "$TMP_DIR" >/dev/null
"$ROOT/target/debug/bijux" dag validate "$TMP_DIR/dag.json" >/dev/null
"$ROOT/target/debug/bijux" dag run "$TMP_DIR/dag.json" --runs-dir "$TMP_DIR/runs" >/dev/null || true
"$ROOT/target/debug/bijux" dag status "$TMP_DIR/runs" >/dev/null || true

echo "post-release minimal workflow completed"
