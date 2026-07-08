from __future__ import annotations

import json
from pathlib import Path

import pytest

from bijux_cli_py import dag_command_json, dag_post_install_diagnostics, load_dag_graph
from bijux_cli_py._exceptions import InternalError, ValidationError


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _write_runtime_stub(path: Path, payload: dict[str, object]) -> None:
    path.write_text(
        "#!/bin/sh\n"
        f"printf '%s\\n' '{json.dumps(payload, separators=(',', ':'))}'\n",
        encoding="utf-8",
    )
    path.chmod(0o755)


def test_load_dag_graph_reads_example_document() -> None:
    graph = load_dag_graph(
        _workspace_root()
        / "evidence"
        / "dag"
        / "authoring"
        / "examples"
        / "hello.dag.json"
    )

    assert graph["spec"] == "bijux-dag/v0.1"
    assert graph["nodes"][0]["id"] == "const1"


def test_load_dag_graph_rejects_non_object_payload(tmp_path: Path) -> None:
    graph = tmp_path / "not-object.json"
    graph.write_text('["not","a","graph"]\n', encoding="utf-8")

    with pytest.raises(ValidationError):
        load_dag_graph(graph)


def test_dag_command_json_uses_bijux_dag_override(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    runtime = tmp_path / "bijux-dag"
    _write_runtime_stub(
        runtime,
        {
            "ok": True,
            "status": "ok",
            "command": "dag.validate",
            "data": {"runtime": "stub"},
            "diagnostics": [],
            "error": None,
        },
    )
    monkeypatch.setenv("BIJUX_DAG_BIN", str(runtime))

    payload = dag_command_json(["validate", "graph.json"])

    assert payload["ok"] is True
    assert payload["data"]["runtime"] == "stub"


def test_dag_command_json_rejects_non_json_payload(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    runtime = tmp_path / "bijux-dag"
    runtime.write_text("#!/bin/sh\nprintf '%s\\n' 'not-json'\n", encoding="utf-8")
    runtime.chmod(0o755)
    monkeypatch.setenv("BIJUX_DAG_BIN", str(runtime))

    with pytest.raises(InternalError):
        dag_command_json(["validate", "graph.json"])


def test_dag_post_install_diagnostics_reports_runtime_override(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    runtime = tmp_path / "bijux-dag"
    _write_runtime_stub(
        runtime,
        {
            "ok": True,
            "status": "ok",
            "command": "dag.version",
            "data": {"version": "0.4.0"},
            "diagnostics": [],
            "error": None,
        },
    )
    monkeypatch.setenv("BIJUX_DAG_BIN", str(runtime))

    diagnostics = dag_post_install_diagnostics()

    assert diagnostics["runtime_binary"] == str(runtime.resolve())
    assert diagnostics["warnings"] == []
