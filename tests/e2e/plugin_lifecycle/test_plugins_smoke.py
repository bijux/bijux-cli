# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin smoke E2E tests."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess  # noqa: S603
import sys

import pytest

from tests.regression.test_functional import cli

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def test_plugin_install_load_run(tmp_path: Path) -> None:
    """Install a minimal PyPI-style plugin and run its command."""
    pkg_dir = tmp_path / "smoke_pkg"
    pkg_dir.mkdir()
    (pkg_dir / "pyproject.toml").write_text(
        "\n".join(
            [
                "[build-system]",
                'requires = ["setuptools>=61.0", "wheel"]',
                'build-backend = "setuptools.build_meta"',
                "",
                "[project]",
                'name = "bijux-cli-smoke-plugin"',
                'version = "0.1.0"',
                'description = "smoke plugin"',
                'requires-python = ">=3.11"',
                'dependencies = ["bijux-cli>=0.0.0"]',
                "",
                '[project.entry-points."bijux_cli.plugins"]',
                'smoke = "smoke_plugin:app"',
                "",
            ]
        ),
        encoding="utf-8",
    )
    (pkg_dir / "smoke_plugin.py").write_text(
        "\n".join(
            [
                "import typer",
                "",
                "app = typer.Typer()",
                "",
                "@app.command()",
                "def hello() -> None:",
                '    typer.echo("smoke-ok")',
                "",
            ]
        ),
        encoding="utf-8",
    )

    wheel_dir = tmp_path / "wheels"
    wheel_dir.mkdir()
    subprocess.run(  # noqa: S603
        [sys.executable, "-m", "pip", "wheel", str(pkg_dir), "-w", str(wheel_dir)],
        check=True,
        capture_output=True,
        text=True,
    )

    env = {
        "PIP_FIND_LINKS": str(wheel_dir),
        "PIP_NO_INDEX": "1",
        "PIP_DISABLE_PIP_VERSION_CHECK": "1",
    }

    install = cli(
        "plugins",
        "install",
        "bijux-cli-smoke-plugin",
        env=env,
        json_output=True,
        expect_exit_code=None,
    )
    assert install.returncode == 0, install.stderr

    listed = cli("plugins", "list", env=env, json_output=True, expect_exit_code=None)
    assert listed.returncode in (0, 1)
    data = listed.json_out or listed.json_err
    plugins: object = []
    if isinstance(data, dict):
        plugins = data.get("plugins", [])
    elif isinstance(data, list):
        plugins = data
    if isinstance(plugins, str):
        plugins = json.loads(plugins)
    names = {
        p["name"]
        for p in (plugins if isinstance(plugins, list) else [])
        if isinstance(p, dict)
    }
    assert "smoke" in names

    cmd = cli("smoke", "hello", env=env, expect_exit_code=None)
    assert cmd.returncode == 0
    assert "smoke-ok" in cmd.stdout

    uninstall = cli(
        "plugins",
        "uninstall",
        "smoke",
        env=env,
        json_output=True,
        expect_exit_code=None,
    )
    assert uninstall.returncode == 0
