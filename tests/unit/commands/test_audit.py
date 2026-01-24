# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the audit command."""

from __future__ import annotations

import json
from pathlib import Path
import platform
from types import SimpleNamespace
from typing import Any
from unittest.mock import MagicMock, patch

import pytest
import typer
from typer.testing import CliRunner

from bijux_cli.cli.commands.diagnostics.audit import (
    _build_payload,
    _write_output_file,
    audit,
    audit_app,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import ExecutionPolicy, resolve_log_policy
from bijux_cli.infra.contracts import Serializer

runner = CliRunner()


class _BasicSerializer:
    def _normalize(self, obj: Any) -> Any:
        if hasattr(obj, "__dataclass_fields__"):
            data = {}
            for key, value in obj.__dict__.items():
                if value is None:
                    continue
                data[key] = self._normalize(value)
            return data
        if isinstance(obj, dict):
            return {key: self._normalize(value) for key, value in obj.items()}
        if isinstance(obj, list | tuple | set):
            return [self._normalize(value) for value in obj]
        if hasattr(obj, "value"):
            return obj.value
        return obj

    def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
        _ = (fmt, pretty)
        return json.dumps(self._normalize(obj))

    def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

    def loads(self, data: str | bytes, *, fmt: OutputFormat, pretty: bool) -> Any:
        _ = (fmt, pretty)
        return json.loads(data)

    def emit(self, payload: Any, *, fmt: OutputFormat, pretty: bool) -> None:
        _ = (payload, fmt, pretty)
        return None


def _fake_configure_emitter(monkeypatch: pytest.MonkeyPatch, emitter: Any) -> None:
    """Patch DIContainer to return a dummy emitter and serializer."""
    fake = SimpleNamespace(
        resolve=lambda iface: _BasicSerializer() if iface is Serializer else emitter
    )
    monkeypatch.setattr(DIContainer, "current", staticmethod(lambda: fake))


class DummyEmitter:
    """Test emitter: matches Emitter, keeps legacy `emitted`."""

    def __init__(self) -> None:
        self._records: list[dict[str, Any]] = []

    @property
    def emitted(self) -> list[dict[str, Any]]:
        return self._records

    def emit(
        self,
        payload: Any,
        *,
        fmt: str | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "",
        output: str | None = None,
        **context: Any,
    ) -> None:
        rec = {
            "payload": payload,
            "fmt": fmt,
            "pretty": pretty,
            "level": level,
            "message": message,
            "output": output,
        }
        rec.update(context)
        self._records.append(rec)

    def flush(self) -> None:
        pass


def test_build_payload_without_runtime() -> None:
    """Test building the basic audit payload without runtime info."""
    payload = _build_payload(include_runtime=False, dry_run=False)
    assert payload.status == "completed"

    payload = _build_payload(include_runtime=False, dry_run=True)
    assert payload.status == "dry-run"


def test_build_payload_with_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test building the audit payload with runtime info included."""
    monkeypatch.setattr(platform, "python_version", lambda: "3.11.9")
    monkeypatch.setattr(platform, "platform", lambda: "TestOS-99")

    calls: list[str] = []

    def fake_ascii_safe(val: str, name: str) -> str:
        calls.append(name)
        return f"{name}-{val}"

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.ascii_safe", fake_ascii_safe
    )

    payload = _build_payload(include_runtime=True, dry_run=False)
    assert payload.status == "completed"
    assert payload.python == "python_version-3.11.9"
    assert payload.platform == "platform-TestOS-99"
    assert set(calls) == {"python_version", "platform"}


def test_write_output_file_parent_missing(tmp_path: Path) -> None:
    """Error if output dir doesn't exist."""
    emitter = DummyEmitter()
    payload: dict[str, Any] = {}
    out = tmp_path / "nope" / "file.yaml"

    with pytest.raises(OSError, match=r"Output directory does not exist"):
        _write_output_file(
            output_path=out,
            payload=payload,
            emitter=emitter,
            fmt=OutputFormat.YAML,
            pretty=False,
            log_policy=resolve_log_policy(LogLevel.DEBUG),
            dry_run=False,
        )


def test_audit_ascii_env_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that non-ASCII environment variables trigger a structured error."""
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.contains_non_ascii_env", lambda: True
    )
    dummy = DummyEmitter()
    _fake_configure_emitter(monkeypatch, dummy)

    result = runner.invoke(audit_app, [], catch_exceptions=False)
    assert result.exit_code == 3
    err = json.loads(result.stderr)
    assert err["failure"] == "ascii_env"
    assert err["code"] == 3
    assert "Non-ASCII environment" in err["error"]


def test_audit_env_file_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an invalid environment file triggers a structured error."""

    def bad_validate(path: str) -> None:
        raise ValueError("bad config")

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.validate_env_file_if_present",
        bad_validate,
    )
    dummy = DummyEmitter()
    _fake_configure_emitter(monkeypatch, dummy)

    result = runner.invoke(audit_app, [], catch_exceptions=False)
    assert result.exit_code == 3
    err = json.loads(result.stderr)
    assert err["failure"] == "ascii"
    assert err["code"] == 3
    assert "bad config" in err["error"]


def test_audit_unexpected_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an unexpected internal error is handled gracefully."""

    class Bad:
        def resolve(self, iface: Any) -> None:
            raise RuntimeError("boom")

    monkeypatch.setattr(DIContainer, "current", staticmethod(lambda: Bad()))
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.contains_non_ascii_env", lambda: False
    )
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.validate_env_file_if_present",
        lambda v: None,
    )
    monkeypatch.setattr(
        "bijux_cli.cli.core.emit.resolve_serializer", lambda: _BasicSerializer()
    )

    result = runner.invoke(audit_app, [], catch_exceptions=False)
    assert result.exit_code == 1
    err = json.loads(result.stderr)
    assert err["failure"] == "unexpected"
    assert err["code"] == 1
    assert "boom" in err["error"]


def test_audit_write_to_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the successful writing of an audit report to a file."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )

    capture: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command",
        lambda **kw: capture.update(kw),
    )

    out_file = tmp_path / "report.json"
    result = runner.invoke(
        audit_app,
        ["--output", str(out_file), "--log-level", "debug"],
        catch_exceptions=False,
    )
    assert result.exit_code == 0

    assert len(emitter.emitted) == 1
    emit_call = emitter.emitted[0]
    assert emit_call["output"] == str(out_file)
    assert "Audit completed" in emit_call["message"]

    builder = capture["payload_builder"]
    out_payload = builder(False)
    assert out_payload.status == "written"
    assert out_payload.file == str(out_file)
    assert out_payload.python is not None
    assert out_payload.platform is not None


def test_audit_output_file_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an invalid output file path triggers a structured error."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)

    result = runner.invoke(
        audit_app, ["--output", "/no/such/dir/out.json"], catch_exceptions=False
    )
    assert result.exit_code == 2
    err = json.loads(result.stderr)
    assert err["failure"] == "output_file"
    assert err["code"] == 2
    assert "Output directory does not exist" in err["error"]


def test_audit_dry_run_stdout(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the successful execution of a dry-run audit to stdout."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.YAML,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=False,
        ),
    )

    called: dict[str, Any] = {}

    def fake_new_run_command(**kwargs: Any) -> None:
        called.update(kwargs)

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command", fake_new_run_command
    )

    result = runner.invoke(
        audit_app, ["--dry-run", "--format", "yaml"], catch_exceptions=False
    )
    assert result.exit_code == 0

    builder = called["payload_builder"]
    payload = builder(False)
    assert payload.status == "dry-run"

    assert called["command_name"] == "audit"
    assert called["fmt"] == "yaml"
    assert "dry_run" not in called


def test_build_payload_variants() -> None:
    """Test the different variants of the audit payload."""
    p1 = _build_payload(False, False)
    assert p1.status == "completed"
    p2 = _build_payload(False, True)
    assert p2.status == "dry-run"
    p3 = _build_payload(True, False)
    assert p3.status == "completed"
    assert p3.python
    assert p3.platform
    assert p3.python == p3.python.encode("ascii", "ignore").decode()


def test_write_output_file_success(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Emit is called with the expected args when writing succeeds (dry-run)."""
    emitter = DummyEmitter()
    called: dict[str, Any] = {}

    def fake_emit(
        payload: Any,
        *,
        fmt: OutputFormat | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "",
        output: str | None = None,
        **context: Any,
    ) -> None:
        called.update(
            payload=payload,
            fmt=fmt,
            pretty=pretty,
            level=level,
            message=message,
            output=output,
            context=context,
        )

    monkeypatch.setattr(emitter, "emit", fake_emit, raising=False)

    out = tmp_path / "subdir" / "out.json"
    (tmp_path / "subdir").mkdir()

    _write_output_file(
        output_path=out,
        payload={"foo": "bar"},
        emitter=emitter,
        fmt=OutputFormat.JSON,
        pretty=True,
        log_policy=resolve_log_policy(LogLevel.INFO),
        dry_run=True,
    )

    assert called["payload"] == {"foo": "bar"}
    assert called["output"] == str(out)
    assert "dry-run" in called["message"]
    assert called["fmt"] is OutputFormat.JSON
    assert called["pretty"] is True
    assert called["level"] == "info"


def test_write_output_file_os_error(tmp_path: Path) -> None:
    """Raise OSError when parent output directory doesn't exist."""
    out = tmp_path / "does_not" / "exist" / "f.json"
    with pytest.raises(OSError, match="does not exist"):
        _write_output_file(
            output_path=out,
            payload={},
            emitter=DummyEmitter(),
            fmt=OutputFormat.JSON,
            pretty=False,
            log_policy=resolve_log_policy(LogLevel.INFO),
            dry_run=False,
        )


def test_ascii_env_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that non-ASCII environment variables are correctly detected."""
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.contains_non_ascii_env", lambda: True
    )
    result = runner.invoke(audit_app, ["--dry-run"], catch_exceptions=False)
    assert result.exit_code == 3
    err = json.loads((result.stdout or result.stderr).strip())
    assert err["failure"] == "ascii_env"


def test_env_file_validation_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a validation error for an environment file is handled."""
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.validate_env_file_if_present",
        lambda path: (_ for _ in ()).throw(ValueError("bad env")),
    )
    result = runner.invoke(audit_app, ["--dry-run"], catch_exceptions=False)
    assert result.exit_code == 3
    err = json.loads((result.stdout or result.stderr).strip())
    assert err["failure"] == "ascii"
    assert "bad env" in err["error"]


def test_debug_includes_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the debug flag includes runtime info in the payload."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )
    captured: dict[str, Any] = {}

    def fake_nrc(**kw: Any) -> None:
        captured.update(kw)

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command", fake_nrc
    )

    runner.invoke(
        audit_app, ["--dry-run", "--log-level", "debug"], catch_exceptions=False
    )
    builder = captured["payload_builder"]
    payload = builder(True)
    assert payload.status == "dry-run"
    assert payload.python
    assert payload.platform


def test_debug_does_not_force_pretty(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that debug does not override pretty flags."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.DEBUG,
            pretty=False,
            include_runtime=False,
        ),
    )
    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command",
        lambda **kw: captured.update(kw),
    )

    runner.invoke(
        audit_app, ["--dry-run", "--log-level", "debug"], catch_exceptions=False
    )
    assert captured["pretty"] is False


def test_audit_stray_argument() -> None:
    """Test that a stray positional argument is rejected by Click."""
    result = runner.invoke(audit_app, ["--dry-run", "extra"])
    assert result.exit_code != 0
    combined = result.stdout + result.stderr
    assert "No such command 'extra'" in combined


def test_audit_invoked_subcommand_returns(monkeypatch: pytest.MonkeyPatch) -> None:
    """Ensure audit callback exits early when a subcommand is invoked."""
    with patch(
        "bijux_cli.cli.commands.diagnostics.audit.validate_common_flags"
    ) as mock_vcf:

        @audit_app.command("sub")
        def sub_command() -> None:
            print("sub_command_output")

        result = runner.invoke(audit_app, ["sub"])
        assert result.exit_code == 0
        assert "sub_command_output" in result.stdout

        mock_vcf.assert_not_called()


def test_audit_catches_value_error_from_env_validation(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test the specific ValueError handler for env file validation."""
    error_message = "Invalid characters in .env file"
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.validate_env_file_if_present",
        lambda path: (_ for _ in ()).throw(ValueError(error_message)),
    )

    result = runner.invoke(audit_app, [], catch_exceptions=False)

    assert result.exit_code == 3
    err_payload = json.loads(result.stderr)

    assert err_payload["failure"] == "ascii"
    assert err_payload["error"] == error_message
    assert err_payload["code"] == 3


def test_audit_catches_value_error_from_main_logic(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test the ValueError handler in the main audit logic."""
    _fake_configure_emitter(monkeypatch, DummyEmitter())

    error_message = "ASCII conversion failed"
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit._build_payload",
        lambda include_runtime, dry_run: (_ for _ in ()).throw(
            ValueError(error_message)
        ),
    )

    result = runner.invoke(audit_app, [], catch_exceptions=False)

    assert result.exit_code == 3
    err_payload = json.loads(result.stderr)

    assert err_payload["failure"] == "ascii"
    assert err_payload["error"] == error_message
    assert err_payload["code"] == 3


def test_audit_catches_generic_exception_from_main_logic(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test the generic Exception handler for unexpected errors."""
    _fake_configure_emitter(monkeypatch, DummyEmitter())

    error_message = "Something broke unexpectedly"
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit._build_payload",
        lambda include_runtime, dry_run: (_ for _ in ()).throw(KeyError(error_message)),
    )

    result = runner.invoke(audit_app, [], catch_exceptions=False)

    assert result.exit_code == 1
    err_payload = json.loads(result.stderr)

    assert err_payload["failure"] == "unexpected"
    assert f"An unexpected error occurred: '{error_message}'" == err_payload["error"]
    assert err_payload["code"] == 1


def test_audit_output_to_file_with_debug_includes_runtime(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that debug file output includes runtime info in the final payload."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )

    captured_kwargs: dict[str, Any] = {}

    def fake_new_run_command(**kwargs: Any) -> None:
        captured_kwargs.update(kwargs)

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command", fake_new_run_command
    )

    out_file = tmp_path / "report.json"
    result = runner.invoke(
        audit_app, ["--output", str(out_file), "--log-level", "debug"]
    )

    assert result.exit_code == 0
    final_payload_builder = captured_kwargs.get("payload_builder")
    assert final_payload_builder is not None
    final_payload = final_payload_builder(True)

    assert final_payload.status == "written"
    assert final_payload.python is not None
    assert final_payload.platform is not None


def test_audit_output_to_file_with_debug_includes_runtime_in_final_payload(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that debug file output writes a payload with runtime info."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )

    out_file = tmp_path / "report.json"
    result = runner.invoke(
        audit_app, ["--output", str(out_file), "--log-level", "debug"]
    )

    assert result.exit_code == 0
    assert len(emitter.emitted) == 1

    file_emit_call = emitter.emitted[0]
    payload_written_to_file = file_emit_call["payload"]

    assert payload_written_to_file["status"] == "completed"
    assert payload_written_to_file["python"]
    assert payload_written_to_file["platform"]


def test_audit_output_to_file_with_debug(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that debug file output writes the correct payload."""
    emitter = DummyEmitter()
    _fake_configure_emitter(monkeypatch, emitter)
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )

    out_file = tmp_path / "report.json"
    result = runner.invoke(
        audit_app, ["--output", str(out_file), "--log-level", "debug"]
    )

    assert result.exit_code == 0
    assert len(emitter.emitted) == 1

    payload_written_to_file = emitter.emitted[0]["payload"]

    assert payload_written_to_file["status"] == "completed"
    assert payload_written_to_file["python"]
    assert payload_written_to_file["platform"]


def test_audit_stray_argument_error_path(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the stray argument handling logic."""
    mock_emit_error = MagicMock(side_effect=SystemExit(2))
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.emit_error_with_policy",
        mock_emit_error,
    )

    mock_ctx = MagicMock(spec=typer.Context)
    mock_ctx.invoked_subcommand = False
    mock_ctx.args = ["strayarg"]

    with pytest.raises(SystemExit) as excinfo:
        audit(
            ctx=mock_ctx,
            dry_run=False,
            output=None,
            quiet=False,
            fmt="json",
            pretty=True,
            log_level=LogLevel.INFO,
        )

    assert excinfo.value.code == 2

    mock_emit_error.assert_called_once()
    call_args, call_kwargs = mock_emit_error.call_args

    assert call_args[0] == "No such argument: strayarg"
    assert call_kwargs.get("code") == 2
    assert call_kwargs.get("failure") == "args"


def test_debug_file_output_constructs_correct_final_payload(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that debug file output executes the final payload modification."""
    _fake_configure_emitter(monkeypatch, DummyEmitter())
    monkeypatch.setattr(
        "bijux_cli.cli.core.command.current_execution_policy",
        lambda: ExecutionPolicy(
            output_format=OutputFormat.JSON,
            color=ColorMode.AUTO,
            quiet=False,
            log_level=LogLevel.INFO,
            pretty=True,
            include_runtime=True,
        ),
    )

    captured_kwargs: dict[str, Any] = {}

    def fake_new_run_command(**kwargs: Any) -> None:
        captured_kwargs.update(kwargs)

    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command", fake_new_run_command
    )

    out_file = tmp_path / "report.json"
    result = runner.invoke(
        audit_app, ["--output", str(out_file), "--log-level", "debug"]
    )

    assert result.exit_code == 0
    assert "payload_builder" in captured_kwargs
    final_payload = captured_kwargs["payload_builder"](True)

    assert final_payload.status == "written"
    assert final_payload.python is not None
    assert final_payload.platform is not None


def test_audit_written_without_runtime_when_not_debug(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that non-debug file output does not include runtime info."""
    dummy = DummyEmitter()
    _fake_configure_emitter(monkeypatch, dummy)

    captured: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.cli.commands.diagnostics.audit.new_run_command",
        lambda **kw: captured.update(kw),
    )

    out_file = tmp_path / "audit.json"
    result = runner.invoke(
        audit_app, ["--output", str(out_file)], catch_exceptions=False
    )

    assert result.exit_code == 0
    assert "payload_builder" in captured
    builder = captured["payload_builder"]
    payload = builder(False)
    assert payload.status == "written"
    assert payload.file == str(out_file)
    assert payload.python is None
    assert payload.platform is None
