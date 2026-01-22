# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Global CLI flag parsing helpers."""

from __future__ import annotations

import sys
from typing import Any


def parse_global_flags(
    argv: list[str],
) -> tuple[dict[str, Any], list[str], list[dict[str, Any]]]:
    """Parse global CLI flags from argv and return flags + remaining args."""
    help_present = any(flag in ("-h", "--help") for flag in argv)
    flags: dict[str, Any] = {
        "help": False,
        "quiet": False,
        "verbose": False,
        "format": "json",
        "pretty": True,
        "log_level": "info",
        "color": "auto",
        "debug": False,
    }
    errors: list[dict[str, Any]] = []
    retained: list[str] = []
    it = iter(argv)
    for flag in it:
        if flag in ("-h", "--help"):
            flags["help"] = True
            retained.append(flag)
            continue
        if help_present:
            retained.append(flag.lstrip("-"))
            continue
        if flag in ("-d", "--debug"):
            flags["debug"] = True
            continue
        if flag in ("-q", "--quiet"):
            flags["quiet"] = True
        elif flag in ("-v", "--verbose"):
            flags["verbose"] = True
        elif flag in ("-f", "--format"):
            try:
                flags["format"] = next(it)
            except StopIteration:
                errors.append(
                    {
                        "message": "Missing value for --format.",
                        "failure": "missing_argument",
                        "flag": "--format",
                    }
                )
        elif flag == "--log-level":
            try:
                flags["log_level"] = next(it)
            except StopIteration:
                errors.append(
                    {
                        "message": "Missing value for --log-level.",
                        "failure": "missing_argument",
                        "flag": "--log-level",
                    }
                )
        elif flag == "--color":
            try:
                flags["color"] = next(it)
            except StopIteration:
                errors.append(
                    {
                        "message": "Missing value for --color.",
                        "failure": "missing_argument",
                        "flag": "--color",
                    }
                )
        elif flag == "--pretty":
            flags["pretty"] = True
        elif flag == "--no-pretty":
            flags["pretty"] = False
        else:
            retained.append(flag)
    return flags, retained, errors


def apply_parsed_flags(flags: dict[str, Any], retained: list[str]) -> None:
    """Rewrite sys.argv with parsed global flags removed."""
    sys.argv = [sys.argv[0], *retained]


def parse_and_apply_global_flags(
    argv: list[str],
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """Parse and apply global CLI flags to sys.argv."""
    flags, retained, errors = parse_global_flags(argv)
    apply_parsed_flags(flags, retained)
    return flags, errors


__all__ = ["apply_parsed_flags", "parse_and_apply_global_flags", "parse_global_flags"]
