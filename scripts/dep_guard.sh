#!/usr/bin/env bash
set -euo pipefail

fail=0

check_forbidden() {
  local file="$1"
  local dep="$2"
  if rg -n "^${dep}\s*=|\"${dep}\"" "$file" >/dev/null 2>&1; then
    echo "forbidden dependency: ${dep} in ${file}" >&2
    fail=1
  fi
}

check_forbidden crates/bijux_dag_runtime/Cargo.toml bijux_dag_app
check_forbidden crates/bijux_dag_runtime/Cargo.toml bijux_cli
check_forbidden crates/bijux_dag_core/Cargo.toml bijux_dag_runtime
check_forbidden crates/bijux_dag_core/Cargo.toml bijux_dag_artifacts

exit $fail
