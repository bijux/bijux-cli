#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "usage: verify_tag_reproducibility.sh <tag>" >&2
  exit 2
fi

CURRENT_SHA="$(git -C "$ROOT" rev-parse HEAD)"
TAG_SHA="$(git -C "$ROOT" rev-list -n 1 "$TAG")"

if [[ "$CURRENT_SHA" != "$TAG_SHA" ]]; then
  echo "reproducibility check failed: HEAD ($CURRENT_SHA) != tag ($TAG_SHA)"
  exit 1
fi

echo "reproducibility check passed: $TAG -> $TAG_SHA"
