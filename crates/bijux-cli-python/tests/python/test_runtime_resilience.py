from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess

import pytest

from bijux_cli_py import (
    config_resolution_helpers,
    migration_warnings,
    plugin_registry_inspection,
    post_install_diagnostics,
)
from bijux_cli_py._exceptions import (
    InternalError,
    NativeExtensionUnavailable,
    PlatformWheelUnavailable,
    UsageError,
    ValidationError,
)
from bijux_cli_py._facade import (
    ensure_native_extension,
    error_to_exception,
    execution_facade,
    execution_facade_with_status,
)
from bijux_cli_py._facade import (
    version as facade_version,
)
import bijux_cli_py._runtime as runtime


def test_legacy_runtime_warning_is_opt_in() -> None:
    assert migration_warnings() == []
    warnings = migration_warnings(legacy_python_only=True)
    assert any("deprecated" in warning.lower() for warning in warnings)


def test_platform_wheel_unavailable_error(monkeypatch: pytest.MonkeyPatch) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.delenv("BIJUX_BIN", raising=False)
    monkeypatch.setattr(facade, "_workspace_runtime_binaries", lambda: [])
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


def test_workspace_runtime_resolution_wins_over_path_binary(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    import bijux_cli_py._facade as facade

    workspace_bin = tmp_path / "workspace-bijux"
    workspace_bin.write_text('#!/bin/sh\necho \'{"version":"8.8.8"}\'\n', encoding="utf-8")
    workspace_bin.chmod(0o755)

    path_dir = tmp_path / "path-bin"
    path_dir.mkdir()
    path_bin = path_dir / "bijux"
    path_bin.write_text('#!/bin/sh\necho \'{"version":"0.1.3"}\'\n', encoding="utf-8")
    path_bin.chmod(0o755)

    monkeypatch.delenv("BIJUX_BIN", raising=False)
    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setattr(facade, "_workspace_runtime_binaries", lambda: [str(workspace_bin)])
    monkeypatch.setenv("PATH", str(path_dir))

    result = execution_facade(["version"])
    assert '"8.8.8"' in result


def test_bijux_bin_override_must_point_to_executable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setenv("BIJUX_BIN", "/definitely/missing/runtime")

    with pytest.raises(PlatformWheelUnavailable):
        _ = execution_facade_with_status(["version"])


def test_plugin_registry_invalid_json_maps_to_internal_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    class _NativeStub:
        @staticmethod
        def plugin_registry_inspection(_registry_file: str) -> str:
            return "{broken-json"

    monkeypatch.setattr(facade, "native", _NativeStub)
    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", True)

    with pytest.raises(InternalError):
        _ = plugin_registry_inspection("/tmp/registry.json")


def test_plugin_registry_invalid_json_maps_to_internal_error_without_native(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    import bijux_cli_py._facade as facade

    registry = tmp_path / "registry.json"
    registry.write_text("{broken-json", encoding="utf-8")
    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)

    with pytest.raises(InternalError):
        _ = plugin_registry_inspection(str(registry))


def test_config_resolution_helpers_apply_env_overrides_without_native(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    import bijux_cli_py._facade as facade

    home = tmp_path / "home"
    state = home / ".bijux"
    state.mkdir(parents=True)
    (state / ".env").write_text("BIJUXCLI_HISTORY_FILE=config/history.log\n", encoding="utf-8")

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setenv("BIJUXCLI_HISTORY_FILE", str(home / "env-history.log"))

    paths = config_resolution_helpers(str(home))
    assert paths["history_file"] == str(home / "env-history.log")


def test_config_resolution_helpers_reject_malformed_compatibility_file_without_native(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    import bijux_cli_py._facade as facade

    home = tmp_path / "home"
    state = home / ".bijux"
    state.mkdir(parents=True)
    (state / ".env").write_text("BROKEN\n", encoding="utf-8")

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    with pytest.raises(InternalError):
        _ = config_resolution_helpers(str(home))


def test_subprocess_environment_strips_python_runtime_injection_keys(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setenv("PYTHONPATH", "/tmp/poison")
    monkeypatch.setenv("LD_PRELOAD", "/tmp/evil.so")
    monkeypatch.setenv("PATH", os.environ.get("PATH", ""))
    env = facade._sanitized_subprocess_env()
    assert "PYTHONPATH" not in env
    assert "LD_PRELOAD" not in env
    assert "PATH" in env


def test_loader_oserror_is_not_silently_hidden_by_default(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.delenv("BIJUX_PY_ALLOW_NATIVE_OSERROR_FALLBACK", raising=False)
    assert not facade._allow_native_import_fallback(OSError("bad dynamic loader"))


def test_loader_oserror_fallback_can_be_enabled_explicitly(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setenv("BIJUX_PY_ALLOW_NATIVE_OSERROR_FALLBACK", "1")
    assert facade._allow_native_import_fallback(OSError("bad dynamic loader"))


def test_runtime_binary_filenames_include_windows_executable(monkeypatch: pytest.MonkeyPatch) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setattr(facade.os, "name", "nt", raising=False)
    names = facade._runtime_binary_filenames()
    assert "bijux.exe" in names
    assert "bijux" in names


def test_version_api_delegates_to_runtime_outcome(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setattr(
        facade,
        "execution_facade_with_status",
        lambda _argv: facade.ExecutionResult(
            exit_code=0,
            stdout='{"version":"9.9.9"}\n',
            stderr="",
            error_kind=None,
        ),
    )
    assert facade_version().strip() == '{"version":"9.9.9"}'


def test_subprocess_error_classification_uses_stderr_semantics(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setattr(
        facade,
        "_resolve_binary",
        lambda: facade.RuntimeResolution(binary="/tmp/fake-bijux"),
    )
    monkeypatch.setattr(
        runtime.subprocess,
        "run",
        lambda *_args, **_kwargs: subprocess.CompletedProcess(
            args=["/tmp/fake-bijux", "status"],
            returncode=1,
            stdout="",
            stderr="invalid configuration value",
        ),
    )
    result = execution_facade_with_status(["status"])
    assert result.exit_code == 1
    assert result.error_kind == "ValidationError"


def test_subprocess_timeout_is_normalized_to_internal_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setattr(
        facade,
        "_resolve_binary",
        lambda: facade.RuntimeResolution(binary="/tmp/fake-bijux"),
    )

    def _raise_timeout(*_args: object, **_kwargs: object) -> object:
        raise subprocess.TimeoutExpired(cmd=["/tmp/fake-bijux", "status"], timeout=1)

    monkeypatch.setattr(runtime.subprocess, "run", _raise_timeout)

    result = execution_facade_with_status(["status"])
    assert result.exit_code == 1
    assert result.error_kind == "InternalError"
    assert "timed out" in result.stderr


def test_command_tree_fallback_includes_warning_instead_of_stale_namespaces(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.setattr(facade, "NATIVE_AVAILABLE", False)
    monkeypatch.setattr(
        facade,
        "execution_facade_with_status",
        lambda _argv: facade.ExecutionResult(
            exit_code=2,
            stdout="",
            stderr="unknown namespace: inspect",
            error_kind="UsageError",
        ),
    )

    payload = json.loads(facade.command_tree_introspection())
    assert payload["root"] == "bijux"
    assert payload["namespaces"] == []
    assert payload["source"] == "fallback-empty"
    assert "warning" in payload


def test_strict_native_import_defaults_to_disabled_without_explicit_opt_in(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.delenv("BIJUX_PY_STRICT_IMPORT", raising=False)
    monkeypatch.setenv("CI", "1")
    monkeypatch.delenv("BIJUX_ENV", raising=False)
    assert not facade._strict_native_import_enabled()


def test_strict_native_import_is_enabled_by_explicit_environment_contract(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli_py._facade as facade

    monkeypatch.delenv("BIJUX_PY_STRICT_IMPORT", raising=False)
    monkeypatch.setenv("BIJUX_ENV", "ci")
    assert facade._strict_native_import_enabled()
