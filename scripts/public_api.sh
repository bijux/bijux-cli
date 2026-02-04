#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-public-api >/dev/null 2>&1; then
  echo "cargo-public-api not installed; skipping" >&2
  exit 0
fi

mkdir -p docs/api

for crate in bijux_dag_core bijux_dag_artifacts bijux_dag_runtime bijux_dag_app; do
  out="docs/api/${crate}.txt"
  tmp="${out}.tmp"
  cargo public-api -p "${crate}" > "${tmp}"
  if [[ -f "${out}" ]]; then
    if ! diff -u "${out}" "${tmp}"; then
      echo "public API changed for ${crate}" >&2
      exit 1
    fi
  else
    mv "${tmp}" "${out}"
    echo "wrote baseline ${out}" >&2
    continue
  fi
  rm -f "${tmp}"
  echo "${crate} OK" >&2
 done
