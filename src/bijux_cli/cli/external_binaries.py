# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Safe execution helpers for delegated product binaries."""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
from pathlib import Path
import re
import shutil
import subprocess  # nosec B404
from typing import NamedTuple

import typer

_ALLOWED_PRODUCT_BINARIES = frozenset({"bijux-atlas", "bijux-dev-atlas"})
_ENV_ALLOWED_PRODUCT_BINS = "BIJUXCLI_ALLOWED_PRODUCT_BINS"
_ENV_DEV_MODE = "BIJUX_DEV_MODE"
_ENV_PRODUCT_BIN_DIR = "BIJUXCLI_PRODUCT_BIN_DIR"
_ENV_PRODUCT_BIN_DIRS = "BIJUXCLI_PRODUCT_BIN_DIRS"
_ENV_PRODUCT_BIN_PRECEDENCE = "BIJUXCLI_PRODUCT_BIN_PRECEDENCE"
_ENV_ENFORCE_MAJOR_MATCH = "BIJUXCLI_ENFORCE_PRODUCT_MAJOR_MATCH"

_PRODUCT_BINARY_REQUIREMENTS = {
    "atlas": ("bijux-atlas", "bijux-dev-atlas"),
}


class ProductBinaryProbe(NamedTuple):
    """Resolved product binary status and optional version information."""

    binary: str
    path: str | None
    version: str | None
    compatible_major: bool | None


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
    resolved = resolve_binary_path(bin_name)
    if resolved is not None:
        return resolved
    _emit_error(
        code="binary_not_found",
        message=f"binary {bin_name!r} was not found in configured product bin dirs or PATH",
        exit_code=127,
    )


def _configured_product_bin_dirs() -> list[Path]:
    dirs: list[Path] = []
    single = os.getenv(_ENV_PRODUCT_BIN_DIR, "").strip()
    multi = os.getenv(_ENV_PRODUCT_BIN_DIRS, "").strip()
    if single:
        dirs.append(Path(single))
    if multi:
        for value in multi.split(","):
            candidate = value.strip()
            if candidate:
                dirs.append(Path(candidate))
    deduped: list[Path] = []
    for path in dirs:
        if path not in deduped:
            deduped.append(path)
    return deduped


def _iter_candidate_paths(bin_name: str) -> list[Path]:
    configured = _configured_product_bin_dirs()
    cwd = Path.cwd()
    local_dirs = [cwd / "bin", cwd / "artifacts"]
    precedence = os.getenv(_ENV_PRODUCT_BIN_PRECEDENCE, "extra-first").strip().lower()
    if precedence not in {"extra-first", "path-first"}:
        precedence = "extra-first"

    candidates: list[Path] = []
    if precedence == "extra-first":
        candidates.extend(configured)
        candidates.extend(local_dirs)
    else:
        candidates.extend(local_dirs)
        candidates.extend(configured)
    return [path / bin_name for path in candidates]


def resolve_binary_path(bin_name: str) -> str | None:
    """Resolve a product binary path using configured bin dirs and PATH."""
    for candidate in _iter_candidate_paths(bin_name):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    resolved = shutil.which(bin_name)
    return resolved


def required_product_binaries() -> dict[str, tuple[str, ...]]:
    """Return required binary names per product."""
    return dict(_PRODUCT_BINARY_REQUIREMENTS)


def _extract_semver_major(text: str) -> int | None:
    match = re.search(r"(\\d+)\\.(\\d+)\\.(\\d+)", text)
    if not match:
        return None
    return int(match.group(1))


def probe_binary_version(path: str) -> str | None:
    """Probe `<binary> --version` and return the first output line."""
    try:
        proc = subprocess.run(  # noqa: S603 # nosec B603
            [path, "--version"],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError:
        return None
    if proc.returncode != 0:
        return None
    line = (proc.stdout or proc.stderr or "").splitlines()
    if not line:
        return None
    return line[0].strip() or None


def probe_product_binaries(
    product: str,
    *,
    host_version: str | None = None,
) -> list[ProductBinaryProbe]:
    """Resolve and probe all required binaries for a product."""
    required = _PRODUCT_BINARY_REQUIREMENTS.get(product, ())
    probes: list[ProductBinaryProbe] = []
    enforce_major = os.getenv(_ENV_ENFORCE_MAJOR_MATCH, "0") == "1"
    host_major = _extract_semver_major(host_version or "") if host_version else None

    for binary in required:
        path = resolve_binary_path(binary)
        version = probe_binary_version(path) if path else None
        compatible_major: bool | None = None
        if enforce_major and host_major is not None and version is not None:
            target_major = _extract_semver_major(version)
            compatible_major = target_major == host_major if target_major is not None else False
        probes.append(
            ProductBinaryProbe(
                binary=binary,
                path=path,
                version=version,
                compatible_major=compatible_major,
            )
        )
    return probes


def run_external(binary: ExternalBinaryCommand, args: list[str]) -> int:
    """Run a delegated product binary with strict allowlist and passthrough args."""

    bin_name = _validate_binary_name(binary.bin_name)
    _assert_binary_allowed(bin_name)
    bin_path = _resolve_binary_path(bin_name)
    if os.getenv(_ENV_ENFORCE_MAJOR_MATCH, "0") == "1":
        from bijux_cli.core.version import __version__ as host_version

        probe = probe_product_binaries("atlas", host_version=host_version)
        by_name = {item.binary: item for item in probe}
        current = by_name.get(bin_name)
        if current is not None and current.compatible_major is False:
            _emit_error(
                code="binary_version_incompatible",
                message=(
                    f"binary {bin_name!r} major version is incompatible with host "
                    f"bijux-cli {host_version}"
                ),
                exit_code=2,
            )

    env = os.environ.copy()
    if binary.env_passthrough:
        env = {key: value for key, value in env.items() if key in set(binary.env_passthrough)}

    proc = subprocess.run([bin_path, *args], check=False, env=env)  # noqa: S603 # nosec B603
    return proc.returncode


__all__ = [
    "ExternalBinaryCommand",
    "ProductBinaryProbe",
    "probe_product_binaries",
    "required_product_binaries",
    "resolve_binary_path",
    "run_external",
]
