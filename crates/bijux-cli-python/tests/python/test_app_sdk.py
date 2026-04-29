from __future__ import annotations

import io
import json
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

from bijux_cli_py import (
    CompatibilityWindow,
    build_python_mount_manifest,
    compatibility_report,
    run_json_app,
    success,
)


def _fixture(name: str) -> str:
    return (
        Path(__file__).resolve().parent / "fixtures" / name
    ).read_text(encoding="utf-8").strip()


def test_success_envelope_matches_checked_fixture() -> None:
    rendered = success(
        {"value": 1},
        command=["sample"],
        timestamp="2026-01-01T00:00:00Z",
    ).stdout.strip()
    assert rendered == _fixture("app_sdk_success_envelope.json")


def test_build_python_mount_manifest_uses_module_and_function_fields() -> None:
    manifest = build_python_mount_manifest(
        namespace="sample",
        display_name="Sample App",
        module="sample_app.cli",
        function="main",
        summary="Sample mounted app",
        aliases=["samp"],
        capabilities=["json_output"],
        version="0.1.0",
        compatibility=CompatibilityWindow("0.3.0", "1.0.0"),
    )

    assert manifest["entrypoint"]["module"] == "sample_app.cli"
    assert manifest["entrypoint"]["function"] == "main"
    assert manifest["control_entrypoint"]["module"] == "sample_app.cli"
    assert manifest["compatibility"]["min_cli_version"] == "0.3.0"


def test_compatibility_report_marks_out_of_window_hosts() -> None:
    report = compatibility_report(
        "0.3.0",
        "0.4.0",
        host_cli_version="0.4.1",
    )
    assert report["compatible"] is False
    assert report["reasons"]


def test_run_json_app_redirects_logs_to_stderr_and_keeps_stdout_json_clean() -> None:
    def main(argv: list[str]):
        print(f"log:{argv[0]}")
        return {"argv": argv}

    stdout = io.StringIO()
    stderr = io.StringIO()
    with redirect_stdout(stdout), redirect_stderr(stderr):
        exit_code = run_json_app(main, argv=["inspect"], command=["sample"])

    assert exit_code == 0
    payload = json.loads(stdout.getvalue())
    assert payload["status"] == "ok"
    assert payload["data"]["argv"] == ["inspect"]
    assert "log:inspect" in stderr.getvalue()
