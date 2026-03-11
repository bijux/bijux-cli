from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11
    import tomli as tomllib  # type: ignore[no-redef]


def _project_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _load_pyproject() -> dict[str, object]:
    pyproject = _project_root() / "pyproject.toml"
    return tomllib.loads(pyproject.read_text(encoding="utf-8"))


def _runtime_binary() -> str:
    override = os.environ.get("BIJUX_BIN")
    if override:
        return override

    for name in ("bijux-rs", "bijux"):
        resolved = shutil.which(name)
        if resolved:
            return resolved

    raise RuntimeError("bijux runtime binary not found")


def test_script_entrypoint_name_and_target_are_stable() -> None:
    pyproject = _load_pyproject()
    scripts = pyproject["project"]["scripts"]
    assert scripts == {"bijux": "bijux_cli_py.cli:main"}


def test_maturin_module_name_matches_package_layout() -> None:
    pyproject = _load_pyproject()
    module_name = pyproject["tool"]["maturin"]["module-name"]
    assert module_name == "bijux_cli_py._native"


def test_python_dash_m_and_binary_help_parity() -> None:
    runtime = _runtime_binary()
    direct = subprocess.run([runtime, "--help"], capture_output=True, text=True, check=False)
    env = os.environ.copy()
    package_root = _project_root() / "python"
    env["PYTHONPATH"] = str(package_root)
    module = subprocess.run(
        [sys.executable, "-m", "bijux_cli_py", "--help"],
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )

    assert direct.returncode == module.returncode
    assert direct.stdout.strip() == module.stdout.strip()


def test_project_metadata_is_consistent_for_wheel_builds() -> None:
    pyproject = _load_pyproject()
    project = pyproject["project"]
    assert project["name"] == "bijux-cli"
    assert project["version"] == "0.1.0"
    assert project["requires-python"] == ">=3.9"
