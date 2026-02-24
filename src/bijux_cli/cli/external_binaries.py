# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Safe execution helpers for delegated product binaries."""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
import shutil
import subprocess  # nosec B404

import typer

_ALLOWED_PRODUCT_BINARIES = frozenset({"bijux-atlas", "bijux-dev-atlas"})
_ENV_ALLOWED_PRODUCT_BINS = "BIJUXCLI_ALLOWED_PRODUCT_BINS"
_ENV_DEV_MODE = "BIJUX_DEV_MODE"


@dataclass(frozen=True)
class ExternalBinaryCommand:
    """Configuration for a routed external product binary."""

    bin_name: str
    description: str
    allowlist_key: str
    env_passthrough: tuple[str, ...] = ()


def _emit_error(*, code: str, message: str, exit_code: int) -> None:
    payload = {"code": code, "message": message}
    typer.echo(json.dumps(payload, sort_keys=True), err=True)
    raise typer.Exit(exit_code)


def _validate_binary_name(bin_name: str) -> str:
    if not bin_name or Path(bin_name).name != bin_name or any(ch in bin_name for ch in "/\\"):
        _emit_error(
            code="invalid_binary_name",
            message=f"binary name must be a basename: {bin_name!r}",
            exit_code=2,
        )
    return bin_name


def _allowed_product_binaries() -> set[str]:
    explicit = os.getenv(_ENV_ALLOWED_PRODUCT_BINS, "").strip()
    if explicit:
        return {token.strip() for token in explicit.split(",") if token.strip()}

    if os.getenv(_ENV_DEV_MODE, "0") == "1":
        return set(_ALLOWED_PRODUCT_BINARIES)

    return set()


def _assert_binary_allowed(bin_name: str) -> None:
    allowed = _allowed_product_binaries()
    if bin_name not in allowed:
        _emit_error(
            code="binary_not_allowed",
            message=(
                f"binary {bin_name!r} is not allowed; set {_ENV_ALLOWED_PRODUCT_BINS} "
                "or enable BIJUX_DEV_MODE=1 for local development"
            ),
            exit_code=2,
        )


def _resolve_binary_path(bin_name: str) -> str:
    resolved = shutil.which(bin_name)
    if resolved:
        return resolved

    cwd = Path.cwd()
    for rel in ("bin", "artifacts"):
        candidate = cwd / rel / bin_name
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)

    _emit_error(
        code="binary_not_found",
        message=f"binary {bin_name!r} was not found on PATH or local bin/artifacts",
        exit_code=127,
    )


def run_external(binary: ExternalBinaryCommand, args: list[str]) -> int:
    """Run a delegated product binary with strict allowlist and passthrough args."""

    bin_name = _validate_binary_name(binary.bin_name)
    _assert_binary_allowed(bin_name)
    bin_path = _resolve_binary_path(bin_name)

    env = os.environ.copy()
    if binary.env_passthrough:
        env = {key: value for key, value in env.items() if key in set(binary.env_passthrough)}

    proc = subprocess.run([bin_path, *args], check=False, env=env)  # noqa: S603 # nosec B603
    return proc.returncode


__all__ = [
    "ExternalBinaryCommand",
    "run_external",
]
