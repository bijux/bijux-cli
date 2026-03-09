from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

from bijux_cli_py import (
    command_tree_introspection,
    config_resolution_helpers,
    execution_facade,
    install_path_helpers,
    output_envelope_model,
    plugin_registry_inspection,
    version,
)


def _runtime_binary() -> str:
    override = os.environ.get("BIJUX_BIN")
    if override:
        return override

    for name in ("bijux-rs", "bijux"):
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
