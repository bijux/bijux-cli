from __future__ import annotations

import os
import shutil
import subprocess
import sys
import json
from pathlib import Path
import tempfile

from bijux_cli_py import (
    check_embedded_binary_compatibility,
    check_python_runtime_supported,
    command_tree_introspection,
    config_resolution_helpers,
    deprecated_version_api,
    execution_facade_with_status,
    execution_facade,
    get_command_tree,
    get_version,
    install_path_helpers,
    migration_warnings,
    output_envelope_model,
    path_ambiguity_detection_message,
    post_install_diagnostics,
    plugin_registry_inspection,
    run_cli,
    side_by_side_install_report,
    simulate_pip_uninstall_cleanup,
    simulate_pip_upgrade_preserves_state,
    version,
)


def _runtime_binary() -> str:
    override = os.environ.get("BIJUX_BIN")
    if override:
        return override

    for name in ("bijux", "bijux-rs"):
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
    assert "status" in output_envelope_model()
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
    assert deprecated_version_api() == version()


def test_runtime_support_and_migration_warnings() -> None:
    assert check_python_runtime_supported((3, 10))
    assert not check_python_runtime_supported((3, 8))
    assert migration_warnings(legacy_python_only=True)


def test_post_install_diagnostics_and_binary_compatibility() -> None:
    diagnostics = post_install_diagnostics()
    assert "runtime_supported" in diagnostics
    assert "warnings" in diagnostics
    assert check_embedded_binary_compatibility(version())


def test_pip_uninstall_cleanup_simulation() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        pkg = root / "bijux_cli_py"
        entry = root / "bin" / "bijux"
        pkg.mkdir(parents=True, exist_ok=True)
        (pkg / "__init__.py").write_text("# stub", encoding="utf-8")
        entry.parent.mkdir(parents=True, exist_ok=True)
        entry.write_text("#!/bin/sh\n", encoding="utf-8")
        report = simulate_pip_uninstall_cleanup(str(root))
        assert report["site_package_removed"]
        assert report["entrypoint_removed"]


def test_pip_upgrade_preserves_state_simulation() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        home = Path(tmp)
        bijux = home / ".bijux"
        (bijux / ".plugins").mkdir(parents=True, exist_ok=True)
        (bijux / ".env").write_text("A=B\n", encoding="utf-8")
        (bijux / ".history").write_text("[]\n", encoding="utf-8")
        report = simulate_pip_upgrade_preserves_state(str(home))
        assert report["config_preserved"]
        assert report["history_preserved"]
        assert report["plugins_preserved"]


def test_side_by_side_install_and_path_ambiguity_reporting() -> None:
    report = side_by_side_install_report("/usr/local/bin/bijux", "/opt/homebrew/bin/bijux")
    assert report.has_ambiguity
    assert "Multiple bijux binaries" in report.message
    message = path_ambiguity_detection_message(["/usr/local/bin/bijux", "/usr/local/bin/bijux"])
    assert not message.has_ambiguity
