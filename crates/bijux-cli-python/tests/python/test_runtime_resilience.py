from __future__ import annotations

import os
import json

import pytest

from bijux_cli_py import migration_warnings, post_install_diagnostics
from bijux_cli_py._exceptions import NativeExtensionUnavailable, PlatformWheelUnavailable
from bijux_cli_py._facade import (
    ensure_native_extension,
    error_to_exception,
    execution_facade_with_status,
)
from bijux_cli_py._exceptions import InternalError, UsageError, ValidationError


def test_legacy_runtime_warning_is_opt_in() -> None:
    assert migration_warnings() == []
    warnings = migration_warnings(legacy_python_only=True)
    assert any("deprecated" in warning.lower() for warning in warnings)


def test_platform_wheel_unavailable_error(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv("BIJUX_BIN", raising=False)
    monkeypatch.setenv("PATH", "")
    with pytest.raises(PlatformWheelUnavailable):
        _ = execution_facade_with_status(["version"])


def test_native_extension_failure_message_is_clear() -> None:
    try:
        ensure_native_extension()
    except NativeExtensionUnavailable as exc:
        assert "Rust extension failed to load" in str(exc)


def test_post_install_diagnostics_contains_runtime_binary_or_warning(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("PATH", os.environ.get("PATH", ""))
    diagnostics = post_install_diagnostics()
    assert "runtime_binary" in diagnostics
    assert "warnings" in diagnostics


def test_native_execution_outcome_preserves_exit_code(monkeypatch: pytest.MonkeyPatch) -> None:
    import bijux_cli_py._facade as facade

    class _NativeStub:
        @staticmethod
        def execution_outcome(_argv: list[str]) -> str:
            return json.dumps(
                {
                    "exit_code": 2,
                    "stdout": "",
                    "stderr": "usage failure",
                    "error_kind": "UsageError",
                }
            )

    monkeypatch.setattr(facade, "native", _NativeStub)
    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", True)
    result = execution_facade_with_status(["ghost"])
    assert result.exit_code == 2
    assert result.error_kind == "UsageError"


def test_error_to_exception_maps_typed_python_exceptions() -> None:
    assert isinstance(error_to_exception({"error_kind": "UsageError", "message": "x"}), UsageError)
    assert isinstance(
        error_to_exception({"error_kind": "ValidationError", "message": "x"}),
        ValidationError,
    )
    assert isinstance(error_to_exception({"error_kind": "InternalError", "message": "x"}), InternalError)
