# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""End-to-end plugin lifecycle test."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess  # noqa: S603
import sys
from typing import Any, cast

import pytest

from tests.e2e.conftest import run_cli

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _write_plugin_package(pkg_dir: Path, name: str, plugin: str) -> None:
    pkg_dir.mkdir(parents=True, exist_ok=True)
    (pkg_dir / "pyproject.toml").write_text(
        "\n".join(
            [
                "[build-system]",
                'requires = ["setuptools>=61.0", "wheel"]',
                'build-backend = "setuptools.build_meta"',
                "",
                "[project]",
                f'name = "{name}"',
                'version = "0.1.0"',
                'description = "lifecycle plugin"',
                'requires-python = ">=3.11"',
                'dependencies = ["bijux-cli>=0.0.0"]',
                "",
                '[project.entry-points."bijux_cli.plugins"]',
                f'{plugin} = "lifecycle_plugin:app"',
                "",
            ]
        ),
        encoding="utf-8",
    )
    (pkg_dir / "lifecycle_plugin.py").write_text(
        "\n".join(
            [
                "import typer",
                "",
                "app = typer.Typer()",
                "",
                "@app.command()",
                "def hello() -> None:",
                '    typer.echo("lifecycle-ok")',
                "",
            ]
        ),
        encoding="utf-8",
    )


def _build_wheel(pkg_dir: Path, wheel_dir: Path) -> None:
    wheel_dir.mkdir(parents=True, exist_ok=True)
    subprocess.run(  # noqa: S603
        [sys.executable, "-m", "pip", "wheel", str(pkg_dir), "-w", str(wheel_dir)],
        check=True,
        capture_output=True,
        text=True,
    )


def _load_payload(text: str) -> dict[str, object]:
    data: Any = json.loads(text)
    if not isinstance(data, dict):
        raise AssertionError("Expected JSON object payload")
    return cast(dict[str, object], data)


def test_plugin_lifecycle_install_list_info_check_uninstall(
    tmp_path: Path,
) -> None:
    package_name = "bijux-cli-lifecycle-plugin"
    plugin_name = "lifecycle"
    pkg_dir = tmp_path / "pkg"
    wheel_dir = tmp_path / "wheels"
    plugins_dir = tmp_path / "plugins"

    _write_plugin_package(pkg_dir, package_name, plugin_name)
    _build_wheel(pkg_dir, wheel_dir)

    env = {
        "PIP_FIND_LINKS": str(wheel_dir),
        "PIP_NO_INDEX": "1",
        "PIP_DISABLE_PIP_VERSION_CHECK": "1",
        "BIJUXCLI_PLUGINS_DIR": str(plugins_dir),
    }

    install = run_cli(["plugins", "install", package_name], env=env)
    assert install.returncode == 0, install.stderr
    install_payload = _load_payload(install.stdout)
    assert install_payload.get("status") == "installed"
    plugins = install_payload.get("plugins")
    assert isinstance(plugins, list)
    assert plugin_name in plugins

    listed = run_cli(["plugins", "list", "--format", "json"], env=env)
    assert listed.returncode == 0, listed.stderr
    listed_payload = _load_payload(listed.stdout)
    listed_plugins = listed_payload.get("plugins")
    assert isinstance(listed_plugins, list)
    names = {p.get("name") for p in listed_plugins if isinstance(p, dict)}
    assert plugin_name in names

    info = run_cli(["plugins", "info", plugin_name, "--format", "json"], env=env)
    assert info.returncode == 0, info.stderr
    info_payload = _load_payload(info.stdout)
    assert info_payload.get("name") == plugin_name
    assert info_payload.get("package") == package_name

    checked = run_cli(["plugins", "check", plugin_name, "--format", "json"], env=env)
    assert checked.returncode == 1
    check_payload = _load_payload(checked.stderr or checked.stdout)
    assert check_payload.get("failure") == "health_unavailable"

    uninstall = run_cli(
        ["plugins", "uninstall", plugin_name, "--format", "json"], env=env
    )
    assert uninstall.returncode == 0, uninstall.stderr
    uninstall_payload = _load_payload(uninstall.stdout)
    assert uninstall_payload.get("status") == "uninstalled"
