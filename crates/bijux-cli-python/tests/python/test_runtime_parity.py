from __future__ import annotations

import os
import shutil
import subprocess
import sys
import json
from pathlib import Path

from bijux_cli_py import (
    check_python_runtime_supported,
    command_tree_introspection,
    config_resolution_helpers,
    error_to_exception,
    execution_facade_with_status,
    execution_facade,
    get_command_tree,
    get_version,
    InternalError,
    install_path_helpers,
    migration_warnings,
    post_install_diagnostics,
    plugin_registry_inspection,
    run_cli,
    UsageError,
    ValidationError,
    version,
)


def _runtime_binary() -> str:
    override = os.environ.get("BIJUX_BIN")
    if override:
        return override

    workspace_root = Path(__file__).resolve().parents[4]
    workspace_candidates = [
        workspace_root / "artifacts" / "rust" / "target" / "debug" / "bijux",
        workspace_root / "artifacts" / "rust" / "target" / "release" / "bijux",
        workspace_root / "target" / "debug" / "bijux",
        workspace_root / "target" / "release" / "bijux",
    ]
    for candidate in workspace_candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)

    for name in ("bijux",):
        resolved = shutil.which(name)
        if resolved:
            return resolved
    raise RuntimeError("bijux runtime binary not found")


def test_python_entrypoint_parity_with_runtime_for_version() -> None:
    runtime = _runtime_binary()
    direct = subprocess.run([runtime, "version"], capture_output=True, text=True, check=False)
    wrapper = execution_facade(["version"])

    assert direct.returncode == 0
    assert wrapper.strip() == direct.stdout.strip()


def test_python_facade_normalizes_version_flag_to_version_command() -> None:
    from_flag = execution_facade(["--version"])
    from_command = execution_facade(["version"])
    assert from_flag.strip() == from_command.strip()


def test_python_module_main_parity_with_runtime_for_version() -> None:
    runtime = _runtime_binary()
    direct = subprocess.run([runtime, "version"], capture_output=True, text=True, check=False)
    wrapper = subprocess.run(
        [sys.executable, "-m", "bijux_cli_py", "version"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert direct.returncode == 0
    assert wrapper.returncode == 0
    assert wrapper.stdout.strip() == direct.stdout.strip()


def test_python_facade_apis_are_exposed() -> None:
    assert isinstance(version(), str)
    assert "root" in command_tree_introspection()
    assert "config_file" in config_resolution_helpers(str(Path.home()))
    assert "plugins_dir" in install_path_helpers(str(Path.home()))
    assert plugin_registry_inspection("/tmp/non-existing-registry.json")["version"] == "1"


def test_help_output_parity_with_runtime() -> None:
    runtime = _runtime_binary()
    direct = subprocess.run([runtime, "--help"], capture_output=True, text=True, check=False)
    wrapper = execution_facade(["--help"])
    assert direct.returncode == 0
    assert "Usage:" in wrapper
    assert wrapper.strip() == direct.stdout.strip()


def test_exit_code_and_stderr_parity_for_invalid_command() -> None:
    runtime = _runtime_binary()
    direct = subprocess.run([runtime, "unknown-subcommand"], capture_output=True, text=True, check=False)
    wrapped = execution_facade_with_status(["unknown-subcommand"])
    assert wrapped.exit_code == direct.returncode
    assert isinstance(wrapped.stderr, str)


def test_json_and_yaml_output_parity() -> None:
    runtime = _runtime_binary()
    direct_json = subprocess.run(
        [runtime, "status", "--format", "json"],
        capture_output=True,
        text=True,
        check=False,
    )
    wrapped_json = execution_facade(["status", "--format", "json"])
    assert wrapped_json.strip() == direct_json.stdout.strip()
    assert json.loads(wrapped_json)

    direct_yaml = subprocess.run(
        [runtime, "status", "--format", "yaml"],
        capture_output=True,
        text=True,
        check=False,
    )
    wrapped_yaml = execution_facade(["status", "--format", "yaml"])
    assert wrapped_yaml.strip() == direct_yaml.stdout.strip()
    assert "status:" in wrapped_yaml


def test_plugin_and_repl_startup_parity_smoke() -> None:
    runtime = _runtime_binary()
    direct_plugins = subprocess.run(
        [runtime, "plugins", "list"], capture_output=True, text=True, check=False
    )
    wrapped_plugins = execution_facade(["plugins", "list"])
    assert wrapped_plugins.strip() == direct_plugins.stdout.strip()

    direct_repl = subprocess.run([runtime, "repl"], capture_output=True, text=True, check=False)
    wrapped_repl = execution_facade(["repl"])
    assert wrapped_repl.strip() == direct_repl.stdout.strip()


def test_config_precedence_helpers_and_alias_apis() -> None:
    paths = config_resolution_helpers(str(Path.home()))
    assert paths["config_file"].endswith(".env")
    assert get_version() == version()
    assert "root" in get_command_tree()
    assert isinstance(run_cli(["version"]), str)


def test_runtime_support_and_migration_warnings() -> None:
    assert check_python_runtime_supported((3, 11))
    assert not check_python_runtime_supported((3, 10))
    assert not check_python_runtime_supported((3, 8))
    assert migration_warnings(legacy_python_only=True)


def test_post_install_diagnostics_contract() -> None:
    diagnostics = post_install_diagnostics()
    assert "runtime_supported" in diagnostics
    assert "warnings" in diagnostics


def test_error_to_exception_maps_bridge_error_kinds() -> None:
    usage = error_to_exception({"error_kind": "UsageError", "message": "unknown route"})
    validation = error_to_exception({"error_kind": "ValidationError", "message": "invalid input"})
    internal = error_to_exception({"error_kind": "InternalError", "message": "panic normalized"})
    generic = error_to_exception({"message": "generic"})

    assert isinstance(usage, UsageError)
    assert isinstance(validation, ValidationError)
    assert isinstance(internal, InternalError)
    assert str(generic) == "generic"
