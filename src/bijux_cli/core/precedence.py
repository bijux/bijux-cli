# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Flag/env/config precedence helpers."""

from __future__ import annotations

from collections.abc import Callable
import sys
from typing import Any


def parse_global_flags(
    argv: list[str],
    on_error: Callable[[str, str, dict[str, Any]], None],
) -> tuple[dict[str, Any], list[str]]:
    """Parse global CLI flags from argv and return flags + remaining args."""
    help_present = any(token in ("-h", "--help") for token in argv)
    flags: dict[str, Any] = {
        "help": False,
        "quiet": False,
        "debug": False,
        "verbose": False,
        "format": "json",
        "pretty": True,
        "log_level": "info",
        "color": "auto",
    }
    retained: list[str] = []
    it = iter(argv)
    for token in it:
        if token in ("-h", "--help"):
            flags["help"] = True
            retained.append(token)
            continue
        if help_present:
            retained.append(token.lstrip("-"))
            continue
        elif token in ("-q", "--quiet"):
            flags["quiet"] = True
        elif token in ("-d", "--debug"):
            flags["debug"] = True
            flags["verbose"] = True
        elif token in ("-v", "--verbose"):
            flags["verbose"] = True
        elif token in ("-f", "--format"):
            try:
                flags["format"] = next(it)
            except StopIteration:
                on_error("Missing value for --format.", "missing_argument", flags)
                break
            if flags["format"] not in ("json", "yaml"):
                on_error("Invalid output format.", "invalid_format", flags)
                break
        elif token == "--log-level":
            try:
                flags["log_level"] = next(it)
            except StopIteration:
                on_error("Missing value for --log-level.", "missing_argument", flags)
                break
        elif token == "--color":
            try:
                flags["color"] = next(it)
            except StopIteration:
                on_error("Missing value for --color.", "missing_argument", flags)
                break
            if flags["color"] not in ("auto", "always", "never"):
                on_error("Invalid color mode.", "invalid_color", flags)
                break
        elif token == "--pretty":
            flags["pretty"] = True
        elif token == "--no-pretty":
            flags["pretty"] = False
        else:
            retained.append(token)
    return flags, retained


def apply_parsed_flags(flags: dict[str, Any], retained: list[str]) -> None:
    """Rewrite sys.argv with parsed global flags removed."""
    sys.argv = [sys.argv[0], *retained]


def resolve_output_flags(
    *,
    quiet: bool,
    verbose: bool,
    debug: bool,
    pretty: bool,
    log_level: str = "info",
    color: str = "auto",
) -> dict[str, Any]:
    """Resolve logging/color/pretty flags from a single source of truth."""
    if color not in ("auto", "always", "never"):
        color = "auto"
    include_runtime = (verbose or debug) and not quiet
    effective_pretty = True if (debug and not quiet) else pretty
    if debug:
        effective_log_level = "debug"
    elif quiet:
        effective_log_level = "error"
    else:
        effective_log_level = log_level
    return {
        "include_runtime": include_runtime,
        "pretty": effective_pretty,
        "log_level": effective_log_level,
        "color": color,
    }
