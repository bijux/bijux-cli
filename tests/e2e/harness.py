# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Stateful E2E harness for running the real bijux CLI."""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def _find_bijux_binary() -> Path:
    exe_name = "bijux"
    repo_root = Path(__file__).resolve().parents[2]

    local = repo_root / "bin" / exe_name
    if local.exists():
        return local.resolve()

    sibling = Path(sys.executable).with_name(exe_name)
    if sibling.exists():
        return sibling.resolve()

    for p in repo_root.glob(f".tox/*/*/{exe_name}"):
        if p.is_file():
            return p.resolve()

    if (override := os.getenv("BIJUX_BIN")) and Path(override).is_file():
        return Path(override).resolve()

    if which := shutil.which(exe_name):
        return Path(which).resolve()

    raise FileNotFoundError(
        "Could not locate the 'bijux' binary in "
        "$BIJUX_BIN, interpreter venv, tox envs, PATH, or project ./bin."
    )


@dataclass(frozen=True)
class HarnessResult:
    """Result bundle for CLI subprocess execution."""

    returncode: int
    stdout: str
    stderr: str


class E2EHarness:
    """Persistent, stateful E2E harness for CLI subprocess execution."""

    def __init__(self, root: Path | None = None) -> None:
        """Initialize the harness, optionally reusing a provided workspace."""
        self._tmp: tempfile.TemporaryDirectory[str] | None = None
        if root is None:
            self._tmp = tempfile.TemporaryDirectory()
            self.root = Path(self._tmp.name)
        else:
            self.root = Path(root)
            self.root.mkdir(parents=True, exist_ok=True)
        self.bin = _find_bijux_binary()
        self.env = self._build_env()

    def _build_env(self) -> dict[str, str]:
        env = os.environ.copy()
        env["HOME"] = str(self.root / "home")
        env["XDG_CONFIG_HOME"] = str(self.root / "xdg_config")
        env["XDG_CACHE_HOME"] = str(self.root / "xdg_cache")
        env["XDG_DATA_HOME"] = str(self.root / "xdg_data")
        env["BIJUXCLI_CONFIG"] = str(self.root / ".env")
        env["BIJUXCLI_HISTORY_FILE"] = str(self.root / ".history")
        env["BIJUXCLI_PLUGINS_DIR"] = str(self.root / "plugins")
        env["BIJUXCLI_DOCS_DIR"] = str(self.root / "docs")
        env["BIJUXCLI_TEST_MODE"] = "1"
        env["PYTHONIOENCODING"] = "utf-8"
        return env

    @property
    def config_path(self) -> Path:
        """Return the configured path to the CLI config file."""
        return Path(self.env["BIJUXCLI_CONFIG"])

    @property
    def plugins_dir(self) -> Path:
        """Return the configured path to the plugins directory."""
        return Path(self.env["BIJUXCLI_PLUGINS_DIR"])

    @property
    def history_path(self) -> Path:
        """Return the configured path to the history file."""
        return Path(self.env["BIJUXCLI_HISTORY_FILE"])

    def run(
        self,
        args: Iterable[str],
        *,
        input_data: str | None = None,
        timeout: int = 10,
        extra_env: dict[str, str] | None = None,
    ) -> HarnessResult:
        """Run the CLI with the harness environment."""
        merged = self.env.copy()
        merged.update(extra_env or {})
        proc = subprocess.run(  # noqa: S603
            [str(self.bin), *args],
            input=input_data,
            text=True,
            capture_output=True,
            env=merged,
            timeout=timeout,
        )
        return HarnessResult(proc.returncode, proc.stdout or "", proc.stderr or "")

    def reset(self) -> None:
        """Reset mutable CLI state without destroying the workspace."""
        for path in [
            self.config_path,
            self.history_path,
        ]:
            if path.exists():
                path.unlink()
        if self.plugins_dir.exists():
            shutil.rmtree(self.plugins_dir)

    def cleanup(self) -> None:
        """Clean up temporary resources, if allocated."""
        if self._tmp is not None:
            self._tmp.cleanup()

    def __enter__(self) -> E2EHarness:
        """Return the harness for context manager use."""
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: object | None,
    ) -> None:
        """Tear down temporary state on context exit."""
        self.cleanup()
