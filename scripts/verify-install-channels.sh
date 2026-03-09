#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${root_dir}"

echo "[verify] metadata consistency"
python3.11 scripts/check-package-metadata.py

echo "[verify] rust install contracts"
cargo test -p bijux-cli-install

echo "[verify] cli paths + doctor contract"
cargo test -p bijux-cli-bin --test command_execution

echo "[verify] completed"
