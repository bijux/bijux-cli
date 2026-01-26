# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Benchmarks for critical CLI paths."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess  # noqa: S603
import sys

import pytest
from pytest_benchmark.fixture import BenchmarkFixture  # type: ignore[import-untyped]


def _base_env(tmp_path: Path) -> dict[str, str]:
    return {
        "PYTHONIOENCODING": "utf-8",
        "BIJUXCLI_TEST_MODE": "1",
        "BIJUXCLI_PLUGINS_DIR": str(tmp_path / "plugins"),
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
    }


def _write_dummy_plugin(root: Path, name: str) -> None:
    plugin_dir = root / name
    plugin_dir.mkdir(parents=True, exist_ok=True)
    (plugin_dir / "plugin.py").write_text(
        "import typer\napp = typer.Typer()\n", encoding="utf-8"
    )
    (plugin_dir / "plugin.json").write_text(
        "\n".join(
            [
                "{",
                f'  "name": "{name}",',
                '  "schema_version": "1",',
                '  "version": "0.1.0",',
                '  "bijux_cli_version": ">=0",',
                '  "enabled": true',
                "}",
            ]
        ),
        encoding="utf-8",
    )


@pytest.mark.canary
def test_cli_startup_benchmark(benchmark: BenchmarkFixture, tmp_path: Path) -> None:
    env = _base_env(tmp_path)

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','--help']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 3.0


@pytest.mark.canary
def test_plugin_discovery_benchmark(
    benchmark: BenchmarkFixture, tmp_path: Path
) -> None:
    env = _base_env(tmp_path)
    env["PIP_DISABLE_PIP_VERSION_CHECK"] = "1"

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','plugins','list','--format','json']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 3.0


def test_config_load_benchmark(benchmark: BenchmarkFixture, tmp_path: Path) -> None:
    env = _base_env(tmp_path)
    config_path = Path(env["BIJUXCLI_CONFIG"])
    lines = [f"BIJUXCLI_KEY{i}=value{i}" for i in range(100)]
    config_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','config','list','--format','json']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 2.0


def test_help_fast_path_benchmark(benchmark: BenchmarkFixture, tmp_path: Path) -> None:
    env = _base_env(tmp_path)

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','--help']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 2.5


def test_version_fast_path_benchmark(
    benchmark: BenchmarkFixture, tmp_path: Path
) -> None:
    env = _base_env(tmp_path)

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','version']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 2.5


def test_startup_with_dummy_plugins_benchmark(
    benchmark: BenchmarkFixture, tmp_path: Path
) -> None:
    env = _base_env(tmp_path)
    plugins_dir = Path(env["BIJUXCLI_PLUGINS_DIR"])
    plugins_dir.mkdir(parents=True, exist_ok=True)
    for i in range(25):
        _write_dummy_plugin(plugins_dir, f"dummy_{i}")

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','status']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 4.0


def test_plugin_discovery_warm_cache_benchmark(
    benchmark: BenchmarkFixture, tmp_path: Path
) -> None:
    env = _base_env(tmp_path)
    plugins_dir = Path(env["BIJUXCLI_PLUGINS_DIR"])
    plugins_dir.mkdir(parents=True, exist_ok=True)
    for i in range(10):
        _write_dummy_plugin(plugins_dir, f"warm_{i}")

    def _run() -> None:
        from bijux_cli.plugins.metadata import discover_plugins, invalidate_plugin_cache

        os.environ["BIJUXCLI_PLUGINS_DIR"] = str(plugins_dir)
        invalidate_plugin_cache()
        discover_plugins(strict=False)
        discover_plugins(strict=False)

    result = benchmark(_run)
    assert result is None
    assert benchmark.stats.stats.mean < 1.0


def test_repl_startup_benchmark(benchmark: BenchmarkFixture, tmp_path: Path) -> None:
    env = _base_env(tmp_path)

    def _run() -> subprocess.CompletedProcess[str]:
        return subprocess.run(  # noqa: S603
            [
                sys.executable,
                "-c",
                (
                    "import sys, bijux_cli; "
                    "sys.argv=['bijux','repl']; "
                    "raise SystemExit(bijux_cli.main())"
                ),
            ],
            input="quit\n",
            capture_output=True,
            text=True,
            env=env,
        )

    result = benchmark(_run)
    assert result.returncode in (0, 1, 2)
    assert benchmark.stats.stats.mean < 4.0
