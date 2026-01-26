# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Pure help command logic (intent + payload builders).

This module contains no IO, exit behavior, or policy resolution. It exposes
pure helpers used by the help command runtime.
"""

from __future__ import annotations

from dataclasses import dataclass
import platform as _platform
import sys
import time

import click
import typer

from bijux_cli.cli.core.command import ascii_safe, normalize_format
from bijux_cli.core.enums import LogLevel, OutputFormat
from bijux_cli.core.precedence import EffectiveConfig, OutputConfig

_HUMAN = "human"
_VALID_FORMATS = ("human", "json", "yaml")


@dataclass(frozen=True)
class HelpIntent:
    """Resolved intent for the help command."""

    tokens: list[str]
    fmt_lower: str
    format_value: OutputFormat | None
    error_fmt: OutputFormat
    include_runtime: bool
    pretty: bool
    quiet: bool
    log_level: LogLevel


def _build_help_intent(
    tokens: list[str],
    fmt: str,
    effective: EffectiveConfig,
    output: OutputConfig,
) -> HelpIntent:
    """Build a normalized help intent from raw CLI inputs."""
    fmt_lower = fmt.strip().lower()
    format_value = normalize_format(fmt)
    error_fmt = format_value or OutputFormat.JSON
    return HelpIntent(
        tokens=tokens,
        fmt_lower=fmt_lower,
        format_value=format_value,
        error_fmt=error_fmt,
        include_runtime=output.include_runtime,
        pretty=output.pretty,
        quiet=effective.flags.quiet,
        log_level=output.log_level,
    )


def _find_target_command(
    ctx: typer.Context, path: list[str]
) -> tuple[click.Command, click.Context] | None:
    """Locate the Click command and context for a given command path."""
    root_cmd: click.Command | None = ctx.parent.command if ctx.parent else None
    if not root_cmd:
        return None

    current_cmd: click.Command | None = root_cmd
    current_ctx = click.Context(root_cmd, info_name="bijux")

    for token in path:
        if not isinstance(current_cmd, click.Group):
            return None
        next_cmd = current_cmd.get_command(current_ctx, token)
        if not next_cmd:
            return None
        current_ctx = click.Context(next_cmd, info_name=token, parent=current_ctx)
        current_cmd = next_cmd

    assert current_cmd is not None  # noqa: S101 # nosec: B101
    return current_cmd, current_ctx


def _get_formatted_help(cmd: click.Command, ctx: click.Context) -> str:
    """Return formatted help text for a command."""
    help_text = cmd.get_help(ctx)
    if (
        hasattr(cmd, "context_settings")
        and cmd.context_settings
        and "-h" in cmd.context_settings.get("help_option_names", [])
        and "-h, --help" not in help_text
    ):
        help_text = help_text.replace("--help", "-h, --help")
    return help_text


def _build_help_payload(
    help_text: str, include_runtime: bool, started_at: float
) -> dict[str, object]:
    """Build a structured help payload for JSON/YAML output."""
    payload: dict[str, object] = {"help": help_text}
    if include_runtime:
        return {
            "help": payload["help"],
            "python": ascii_safe(sys.version.split()[0], "python_version"),
            "platform": ascii_safe(_platform.platform(), "platform"),
            "runtime_ms": int((time.perf_counter() - started_at) * 1_000),
        }
    return payload


__all__ = [
    "HelpIntent",
    "_HUMAN",
    "_VALID_FORMATS",
    "_build_help_payload",
    "_build_help_intent",
    "_find_target_command",
    "_get_formatted_help",
]
