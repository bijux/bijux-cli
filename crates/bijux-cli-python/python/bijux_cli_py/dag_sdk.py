"""Python helpers that delegate to the `bijux-dag` runtime."""

from __future__ import annotations

from collections.abc import Sequence
import json
from pathlib import Path
import sys
import tempfile
from typing import Any

from ._exceptions import InternalError, PlatformWheelUnavailable, ValidationError
from ._runtime import (
    ExecutionResult,
    RuntimeResolution,
    classify_process_error_kind,
    resolve_runtime_binary,
    run_subprocess_runtime,
    runtime_timeout_seconds,
    workspace_runtime_binaries,
)


def _workspace_dag_runtime_binaries() -> list[str]:
    return workspace_runtime_binaries(
        module_file=__file__,
        workspace_crate_dir="bijux-dag-cli",
        binary_name="bijux-dag",
    )


def _resolve_dag_binary() -> RuntimeResolution:
    current_entrypoint = sys.argv[0] if sys.argv and sys.argv[0] else None
    return resolve_runtime_binary(
        binary_name="bijux-dag",
        env_key="BIJUX_DAG_BIN",
        workspace_candidates=_workspace_dag_runtime_binaries(),
        current_entrypoint=current_entrypoint,
    )


def _dag_timeout_seconds() -> int:
    return runtime_timeout_seconds("BIJUX_DAG_PY_SUBPROCESS_TIMEOUT")


def _parse_json_result(result: ExecutionResult) -> dict[str, Any]:
    payload_text = result.stdout.strip() or result.stderr.strip()
    if not payload_text:
        raise InternalError(
            "bijux-dag returned no JSON payload; the runtime command may have failed before emitting structured output"
        )
    try:
        payload = json.loads(payload_text)
    except json.JSONDecodeError as exc:
        raise InternalError(
            f"invalid JSON payload from bijux-dag runtime: {exc}"
        ) from exc
    if not isinstance(payload, dict):
        raise InternalError("bijux-dag JSON payload must be a JSON object")
    return payload


def dag_command_json(argv: Sequence[str]) -> dict[str, Any]:
    runtime = _resolve_dag_binary()
    result = run_subprocess_runtime(
        binary=runtime.binary,
        args=["--json", *argv],
        timeout_seconds=_dag_timeout_seconds(),
        classify_error_kind=classify_process_error_kind,
    )
    return _parse_json_result(result)


def _stringify_paths(paths: Sequence[str | Path]) -> list[str]:
    return [str(Path(path).expanduser()) for path in paths]


def _extend_repeatable_flag(
    args: list[str], flag: str, values: Sequence[str] | None
) -> None:
    if not values:
        return
    for value in values:
        args.extend([flag, value])


def load_dag_graph(graph_path: str | Path) -> dict[str, Any]:
    graph = Path(graph_path).expanduser()
    try:
        payload = json.loads(graph.read_text(encoding="utf-8"))
    except OSError as exc:
        raise InternalError(f"failed to read DAG graph '{graph}': {exc}") from exc
    except json.JSONDecodeError as exc:
        raise ValidationError(f"invalid DAG graph JSON in '{graph}': {exc}") from exc
    if not isinstance(payload, dict):
        raise ValidationError(f"DAG graph '{graph}' must decode to a JSON object")
    return payload


def validate_dag_graph(
    *graph_paths: str | Path,
    strict: bool = False,
    explain: bool = False,
    print_fingerprints: bool = False,
) -> dict[str, Any]:
    args = ["validate", *_stringify_paths(graph_paths)]
    if strict:
        args.append("--strict")
    if explain:
        args.append("--explain")
    if print_fingerprints:
        args.append("--print-fingerprints")
    return dag_command_json(args)


def plan_dag_graph(
    *graph_paths: str | Path,
    out: str | Path,
    run_id: str,
    cache_dir: str | Path | None = None,
    absolute_path_policy: str | None = None,
    jobs: int | None = None,
    cpu_budget: int | None = None,
    memory_budget_mb: int | None = None,
    gpu_device_budget: int | None = None,
    resource_capacities: dict[str, int] | None = None,
    from_node: str | None = None,
    to_node: str | None = None,
    select: Sequence[str] | None = None,
    exclude: Sequence[str] | None = None,
    dependency_closure: bool = False,
) -> dict[str, Any]:
    args = [
        "plan",
        "explain",
        *_stringify_paths(graph_paths),
        "--out",
        str(Path(out).expanduser()),
        "--run-id",
        run_id,
    ]
    if cache_dir is not None:
        args.extend(["--cache-dir", str(Path(cache_dir).expanduser())])
    if absolute_path_policy is not None:
        args.extend(["--absolute-path-policy", absolute_path_policy])
    if jobs is not None:
        args.extend(["--jobs", str(jobs)])
    if cpu_budget is not None:
        args.extend(["--cpu-budget", str(cpu_budget)])
    if memory_budget_mb is not None:
        args.extend(["--memory-budget-mb", str(memory_budget_mb)])
    if gpu_device_budget is not None:
        args.extend(["--gpu-device-budget", str(gpu_device_budget)])
    if resource_capacities:
        for name, capacity in resource_capacities.items():
            args.extend(["--resource-capacity", f"{name}={capacity}"])
    if from_node is not None:
        args.extend(["--from-node", from_node])
    if to_node is not None:
        args.extend(["--to-node", to_node])
    _extend_repeatable_flag(args, "--select", select)
    _extend_repeatable_flag(args, "--exclude", exclude)
    if dependency_closure:
        args.append("--dependency-closure")
    return dag_command_json(args)


def run_dag_graph(
    *graph_paths: str | Path,
    out: str | Path,
    run_id: str | None = None,
    graph_inputs: dict[str, Any] | None = None,
    inputs_file: str | Path | None = None,
    jobs: int | None = None,
    cache: str | None = None,
    cache_dir: str | Path | None = None,
    remote_cache_dir: str | Path | None = None,
    materialize_inputs: str | None = None,
    select: Sequence[str] | None = None,
    exclude: Sequence[str] | None = None,
    to_node: str | None = None,
    dependency_closure: bool = False,
    progress: str | None = None,
) -> dict[str, Any]:
    if graph_inputs is not None and inputs_file is not None:
        raise ValueError("graph_inputs and inputs_file are mutually exclusive")

    args = [
        "run",
        *_stringify_paths(graph_paths),
        "--out",
        str(Path(out).expanduser()),
    ]
    if run_id is not None:
        args.extend(["--run-id", run_id])
    if jobs is not None:
        args.extend(["--jobs", str(jobs)])
    if cache is not None:
        args.extend(["--cache", cache])
    if cache_dir is not None:
        args.extend(["--cache-dir", str(Path(cache_dir).expanduser())])
    if remote_cache_dir is not None:
        args.extend(["--remote-cache-dir", str(Path(remote_cache_dir).expanduser())])
    if materialize_inputs is not None:
        args.extend(["--materialize-inputs", materialize_inputs])
    if to_node is not None:
        args.extend(["--to-node", to_node])
    _extend_repeatable_flag(args, "--select", select)
    _extend_repeatable_flag(args, "--exclude", exclude)
    if dependency_closure:
        args.append("--dependency-closure")
    if progress is not None:
        args.extend(["--progress", progress])

    temp_inputs_path: Path | None = None
    if graph_inputs is not None:
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False, encoding="utf-8"
        ) as handle:
            json.dump(graph_inputs, handle)
            handle.write("\n")
            temp_inputs_path = Path(handle.name)
        args.extend(["--inputs-file", str(temp_inputs_path)])
    elif inputs_file is not None:
        args.extend(["--inputs-file", str(Path(inputs_file).expanduser())])

    try:
        return dag_command_json(args)
    finally:
        if temp_inputs_path is not None:
            temp_inputs_path.unlink(missing_ok=True)


def inspect_dag_run(*, run_id: str, root: str | Path) -> dict[str, Any]:
    return dag_command_json(
        ["runs", "inspect", run_id, "--root", str(Path(root).expanduser())]
    )


def query_dag_artifacts(run_dir: str | Path) -> dict[str, Any]:
    return dag_command_json(
        ["artifact", "registry", str(Path(run_dir).expanduser())]
    )


def dag_post_install_diagnostics() -> dict[str, object]:
    warnings_out: list[str] = []
    diagnostics: dict[str, object] = {
        "runtime_binary": None,
        "warnings": warnings_out,
    }
    try:
        diagnostics["runtime_binary"] = _resolve_dag_binary().binary
    except PlatformWheelUnavailable as error:
        warnings_out.append(str(error))
    return diagnostics
