# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Benchmarks for critical CLI paths."""

from __future__ import annotations

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
