#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
repo_root="$(CDPATH='' cd -- "${script_dir}/../.." && pwd)"
artifact_root="${BIJUX_DAG_DEMO_ROOT:-${repo_root}/artifacts/dag-demo}"
run_root="${artifact_root}/runs"
cache_root="${artifact_root}/cache"
graph_path="${repo_root}/evidence/dag/authoring/examples/file-processing-report.dag.json"
source_dir="${repo_root}/evidence/dag/authoring/examples/file-processing-source"
report_title="Repository File Processing Report"
validate_json="${artifact_root}/validate.json"
graph_json="${artifact_root}/graph.json"
cold_json="${artifact_root}/cold-run.json"
explain_json="${artifact_root}/cold-explain.json"
artifact_registry_json="${artifact_root}/artifact-registry.json"
artifact_inspect_json="${artifact_root}/artifact-inspect.json"
warm_json="${artifact_root}/warm-run.json"
replay_json="${artifact_root}/replay.json"
verify_json="${artifact_root}/verify.json"

build_dag_binary() {
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${repo_root}/artifacts/target}"
  cargo build -q -p bijux-dag-cli --bin bijux-dag --locked
  printf '%s\n' "${CARGO_TARGET_DIR}/debug/bijux-dag"
}

if [[ -n "${BIJUX_DAG_BIN:-}" ]]; then
  dag_bin="${BIJUX_DAG_BIN}"
else
  dag_bin="$(build_dag_binary)"
fi

if [[ ! -x "${dag_bin}" ]]; then
  printf 'bijux-dag binary is not executable: %s\n' "${dag_bin}" >&2
  exit 1
fi

rm -rf "${artifact_root}"
mkdir -p "${artifact_root}" "${run_root}" "${cache_root}"

run_json() {
  local label="$1"
  local outfile="$2"
  shift 2
  printf '==> %s\n' "${label}"
  "${dag_bin}" "$@" >"${outfile}"
}

run_json \
  "validate file-processing graph" \
  "${validate_json}" \
  validate --json "${graph_path}"

run_json \
  "inspect graph before execution" \
  "${graph_json}" \
  show-effective-graph --json "${graph_path}"

run_json \
  "execute cold retained run" \
  "${cold_json}" \
  run --json "${graph_path}" \
  --out "${run_root}" \
  --run-id file-processing-cold \
  --cache readwrite \
  --cache-dir "${cache_root}" \
  --input "source_dir=${source_dir}" \
  --input "report_title=${report_title}"

run_json \
  "inspect retained cold run" \
  "${explain_json}" \
  explain --json "${run_root}/run-file-processing-cold"

run_json \
  "list retained artifacts" \
  "${artifact_registry_json}" \
  artifact registry "${run_root}/run-file-processing-cold" --json

run_json \
  "inspect final report artifact" \
  "${artifact_inspect_json}" \
  artifact-inspect --json "${run_root}/run-file-processing-cold" render_report:report.md

run_json \
  "execute warm cache-aware rerun" \
  "${warm_json}" \
  run --json "${graph_path}" \
  --out "${run_root}" \
  --run-id file-processing-warm \
  --cache readwrite \
  --cache-dir "${cache_root}" \
  --input "source_dir=${source_dir}" \
  --input "report_title=${report_title}"

run_json \
  "replay final reporting boundary" \
  "${replay_json}" \
  replay --json \
  --source-run-id file-processing-cold \
  --source-run-root "${run_root}" \
  --out "${run_root}" \
  --run-id file-processing-replay \
  --from-node render_report

run_json \
  "strict-verify replayed run" \
  "${verify_json}" \
  verify --json "${run_root}/run-file-processing-replay" --strict

python3 - "${artifact_root}" <<'PY'
import json
import pathlib
import sys

artifact_root = pathlib.Path(sys.argv[1])
run_root = artifact_root / "runs"
report_path = run_root / "run-file-processing-cold" / "nodes" / "render_report" / "outputs" / "report" / "report.md"

def load(name: str) -> dict:
    return json.loads((artifact_root / name).read_text())

validate = load("validate.json")
graph = load("graph.json")
cold = load("cold-run.json")
explain = load("cold-explain.json")
registry = load("artifact-registry.json")
inspect = load("artifact-inspect.json")
warm = load("warm-run.json")
replay = load("replay.json")
verify = load("verify.json")

assert validate["ok"] is True
assert validate["status"] == "ok"

selected_nodes = set(graph["data"]["selection"]["selected_nodes"])
assert selected_nodes == {
    "validate_files",
    "transform_files",
    "aggregate_metrics",
    "render_report",
}

cold_summary = cold["data"]["summary"]
assert cold_summary["status"] == "success"
assert cold_summary["node_counts"]["success"] == 4
assert pathlib.Path(cold["data"]["run_dir"]).name == "run-file-processing-cold"

assert explain["data"]["status"] == "success"
assert explain["data"]["node_counts"]["success"] == 4

registry_data = registry["data"]
legacy_ids = {item["legacy_artifact_id"] for item in registry_data["artifacts"]}
assert registry_data["total_artifacts"] >= 5
assert "render_report:report.md" in legacy_ids

inspect_data = inspect["data"]
assert inspect_data["legacy_artifact_id"] == "render_report:report.md"
assert inspect_data["promotable"] is True
assert inspect_data["payload_missing"] is False

warm_summary = warm["data"]["summary"]
assert warm_summary["status"] == "success"
assert warm_summary["node_counts"]["cached"] >= 3
assert warm_summary["node_counts"]["success"] == 1

assert replay["data"]["upstream_artifact_verification"]["verified"] is True
assert pathlib.Path(replay["data"]["run_dir"]).name == "run-file-processing-replay"

assert verify["data"]["status"] == "ok"
assert verify["data"]["mode"] == "strict"

assert report_path.exists()
report_text = report_path.read_text()
assert report_text.startswith("# Repository File Processing Report")
assert "Processed files:" in report_text
PY

printf '\n'
printf 'dag demo completed\n'
printf 'artifact root: %s\n' "${artifact_root}"
printf 'cold run: %s\n' "${run_root}/run-file-processing-cold"
printf 'warm run: %s\n' "${run_root}/run-file-processing-warm"
printf 'replay run: %s\n' "${run_root}/run-file-processing-replay"
printf 'report: %s\n' "${run_root}/run-file-processing-cold/nodes/render_report/outputs/report/report.md"
