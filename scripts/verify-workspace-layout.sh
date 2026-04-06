#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

[[ -f Cargo.toml ]] || { echo "missing root Cargo.toml" >&2; exit 1; }
[[ ! -d bijux-dag ]] || { echo "legacy bijux-dag directory must not exist" >&2; exit 1; }
[[ -d configs/dag ]] || { echo "missing DAG config root at configs/dag" >&2; exit 1; }
[[ -d docs/dag ]] || { echo "missing DAG docs root at docs/dag" >&2; exit 1; }
[[ -d evidence/dag ]] || { echo "missing DAG evidence root at evidence/dag" >&2; exit 1; }

required_members=(
  "crates/bijux-cli"
  "crates/bijux-cli-python"
  "crates/bijux-dev-cli"
  "crates/bijux-dag-core"
  "crates/bijux-dag-artifacts"
  "crates/bijux-dag-runtime"
  "crates/bijux-dag-app"
  "crates/bijux-dag-cli"
  "crates/bijux-dag-testkit"
  "crates/bijux-dev-dag"
)

for member in "${required_members[@]}"; do
  if ! grep -Fq "\"$member\"" Cargo.toml; then
    echo "workspace member missing from Cargo.toml: $member" >&2
    exit 1
  fi
  [[ -f "$member/Cargo.toml" ]] || { echo "missing crate manifest: $member/Cargo.toml" >&2; exit 1; }
done

echo "workspace layout verified"
