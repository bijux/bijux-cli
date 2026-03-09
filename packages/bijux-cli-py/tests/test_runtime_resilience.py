from __future__ import annotations

import os

import pytest

from bijux_cli_py import migration_warnings, post_install_diagnostics
from bijux_cli_py._exceptions import NativeExtensionUnavailable, PlatformWheelUnavailable
from bijux_cli_py._facade import ensure_native_extension, execution_facade_with_status


def test_mixed_environment_with_legacy_metadata_warning(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("BIJUXCLI_PY_LEGACY", "1")
    warnings = migration_warnings()
    assert any("legacy" in warning.lower() for warning in warnings)


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
