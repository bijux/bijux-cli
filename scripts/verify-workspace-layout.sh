#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

[[ -f Cargo.toml ]] || { echo "missing root Cargo.toml" >&2; exit 1; }
[[ ! -f bijux-dag/Cargo.toml ]] || { echo "nested bijux-dag/Cargo.toml must not exist" >&2; exit 1; }

required_members=(
  "crates/bijux-cli"
  "crates/bijux-cli-python"
  "crates/bijux-dev-cli"
  "bijux-dag/crates/bijux-dag-core"
  "bijux-dag/crates/bijux-dag-artifacts"
  "bijux-dag/crates/bijux-dag-runtime"
  "bijux-dag/crates/bijux-dag-app"
  "bijux-dag/crates/bijux-dag-cli"
  "bijux-dag/crates/bijux-dag-testkit"
  "bijux-dag/crates/bijux-dev-dag"
)

for member in "${required_members[@]}"; do
  if ! grep -Fq "\"$member\"" Cargo.toml; then
    echo "workspace member missing from Cargo.toml: $member" >&2
    exit 1
  fi
  [[ -f "$member/Cargo.toml" ]] || { echo "missing crate manifest: $member/Cargo.toml" >&2; exit 1; }
done

echo "workspace layout verified"
