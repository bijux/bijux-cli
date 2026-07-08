from __future__ import annotations

import json
from pathlib import Path
import subprocess

import pytest

from bijux_cli_py import (
    inspect_dag_run,
    plan_dag_graph,
    query_dag_artifacts,
    run_dag_graph,
    validate_dag_graph,
)


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _dag_wrapper(path: Path) -> Path:
    root = _workspace_root()
    path.write_text(
        "#!/bin/sh\n"
        f"cd '{root}'\n"
        "exec cargo run -q -p bijux-dag-cli --bin bijux-dag -- \"$@\"\n",
        encoding="utf-8",
    )
    path.chmod(0o755)
    return path


def _hello_graph() -> Path:
    return (
        _workspace_root()
        / "evidence"
        / "dag"
        / "authoring"
        / "examples"
        / "hello.dag.json"
    )


def _direct_dag_json(runtime: Path, *args: str) -> dict[str, object]:
    completed = subprocess.run(
        [str(runtime), "--json", *args],
        capture_output=True,
        text=True,
        check=False,
    )
    payload = json.loads((completed.stdout or completed.stderr).strip())
    assert isinstance(payload, dict)
    return payload


def test_validate_dag_graph_matches_direct_runtime(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = _dag_wrapper(tmp_path / "bijux-dag")
    monkeypatch.setenv("BIJUX_DAG_BIN", str(runtime))

    sdk_payload = validate_dag_graph(_hello_graph())
    direct_payload = _direct_dag_json(runtime, "validate", str(_hello_graph()))

    assert sdk_payload == direct_payload


def test_plan_dag_graph_matches_direct_runtime(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = _dag_wrapper(tmp_path / "bijux-dag")
    monkeypatch.setenv("BIJUX_DAG_BIN", str(runtime))
    out = tmp_path / "plan-runs"

    sdk_payload = plan_dag_graph(_hello_graph(), out=out, run_id="sdk-plan")
    direct_payload = _direct_dag_json(
        runtime,
        "plan",
        "explain",
        str(_hello_graph()),
        "--out",
        str(out),
        "--run-id",
        "sdk-plan",
    )

    assert sdk_payload["ok"] == direct_payload["ok"]
    assert sdk_payload["command"] == direct_payload["command"] == "dag.plan.explain"
    assert sdk_payload["data"]["ordering"] == direct_payload["data"]["ordering"]
    assert (
        sdk_payload["data"]["selection"]["selected_nodes"]
        == direct_payload["data"]["selection"]["selected_nodes"]
    )
    assert sdk_payload["data"]["run_layout"] == direct_payload["data"]["run_layout"]
    assert [node["id"] for node in sdk_payload["data"]["planned_nodes"]] == [
        node["id"] for node in direct_payload["data"]["planned_nodes"]
    ]


def test_run_inspect_and_artifact_queries_match_direct_runtime(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = _dag_wrapper(tmp_path / "bijux-dag")
    monkeypatch.setenv("BIJUX_DAG_BIN", str(runtime))
    out = tmp_path / "run-runs"

    run_payload = run_dag_graph(_hello_graph(), out=out, run_id="sdk-run")
    assert run_payload["ok"] is True
    assert run_payload["command"] == "dag.run"
    assert run_payload["data"]["run_layout"]["run_id"] == "sdk-run"
    assert run_payload["data"]["summary"]["status"] == "success"

    run_dir = out / "run-sdk-run"
    sdk_inspect = inspect_dag_run(run_id="sdk-run", root=out)
    direct_inspect = _direct_dag_json(
        runtime,
        "runs",
        "inspect",
        "sdk-run",
        "--root",
        str(out),
    )
    assert sdk_inspect == direct_inspect

    sdk_artifacts = query_dag_artifacts(run_dir)
    direct_artifacts = _direct_dag_json(
        runtime,
        "artifact",
        "registry",
        str(run_dir),
    )
    assert sdk_artifacts == direct_artifacts
