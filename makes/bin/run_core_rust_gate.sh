#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
shared_gate="${repo_root}/.bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"

if [[ ! -x "${shared_gate}" ]]; then
  echo "shared Rust gate is unavailable: ${shared_gate}" >&2
  exit 1
fi

command_name="${1:-}"
artifact_root="${RS_ARTIFACT_ROOT:-${repo_root}/artifacts/rust}"
run_id="${RS_RUN_ID:-${RUN_ID:-local}}"
target_dir="${RS_TARGET_DIR:-${artifact_root}/target}"

if [[ "${command_name}" == "coverage" ]]; then
  target_dir="${artifact_root}/coverage/${run_id}/target"
fi

case "${command_name}" in
  coverage | test | test-all | test-slow)
    mkdir -p "${target_dir}"
    CARGO_TARGET_DIR="${target_dir}" cargo build --locked -p bijux-dev --bin bijux-dev-cli
    CARGO_TARGET_DIR="${target_dir}" cargo build --locked -p bijux-dag-cli --bin bijux-dag
    export BIJUX_DEV_CLI_BIN="${target_dir}/debug/bijux-dev-cli"
    export BIJUX_DAG_BIN="${target_dir}/debug/bijux-dag"
    ;;
esac

if [[ "${command_name}" == "test" && -n "${NEXTEST_FILTER_EXPR:-}" ]]; then
  export NEXTEST_FAST_EXPR="${NEXTEST_FILTER_EXPR}"
fi

exec "${shared_gate}" "$@"
