# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Pure helpers for the `docs` command (intent + payload builders)."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import asdict, is_dataclass
from pathlib import Path
import platform

from bijux_cli.cli.core.command import ascii_safe
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.version import __version__

CLI_VERSION = __version__


def _default_output_path(base: Path, fmt: OutputFormat) -> Path:
    """Computes the default output file path for a CLI spec."""
    return base / f"spec.{fmt.value}"


def _resolve_output_target(
    out: Path | None, fmt: OutputFormat
) -> tuple[str, Path | None]:
    """Resolves the output target and file path for the CLI spec."""
    if out is None:
        path = _default_output_path(Path.cwd(), fmt)
        return str(path), path
    if str(out) == "-":
        return "-", None
    if out.is_dir():
        path = _default_output_path(out, fmt)
        return str(path), path
    return str(out), out


def _build_spec_payload(include_runtime: bool) -> dict[str, object]:
    """Builds the CLI specification payload."""
    from bijux_cli.cli.commands import list_registered_command_names

    version_str = ascii_safe(CLI_VERSION, "version")
    payload: dict[str, object] = {
        "version": version_str,
        "commands": list_registered_command_names(),
    }
    if include_runtime:
        return {
            "version": payload["version"],
            "commands": payload["commands"],
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def _spec_mapping(spec: Mapping[str, object]) -> dict[str, object]:
    """Convert a spec payload into a mapping for service calls."""
    data = asdict(spec) if is_dataclass(spec) else dict(spec)
    return {key: value for key, value in data.items() if value is not None}


__all__ = [
    "CLI_VERSION",
    "_build_spec_payload",
    "_default_output_path",
    "_resolve_output_target",
    "_spec_mapping",
]
