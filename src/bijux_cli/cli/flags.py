# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Global CLI flag parsing helpers."""

from __future__ import annotations

from typing import Any

from bijux_cli.core.precedence import GlobalCLIConfig


def parse_global_flags(argv: list[str]) -> GlobalCLIConfig:
    """Parse global CLI flags from argv into an immutable config."""
    help_present = any(flag in ("-h", "--help") for flag in argv)
    help_flag = False
    quiet = False
    verbose_level = 0
    fmt = "json"
    pretty = True
    log_level = "info"
    color = "auto"
    json_flag = False
    errors: list[dict[str, Any]] = []
    retained: list[str] = []
    it = iter(argv)
    for flag in it:
        if flag in ("-h", "--help"):
            help_flag = True
            retained.append(flag)
            continue
        if help_present:
            retained.append(flag.lstrip("-"))
            continue
        if flag in ("-q", "--quiet"):
            quiet = True
        elif flag in ("-v", "--verbose"):
            verbose_level += 1
        elif flag == "--log-level":
            try:
                log_level = next(it)
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
                color = next(it)
            except StopIteration:
                errors.append(
                    {
                        "message": "Missing value for --color.",
                        "failure": "missing_argument",
                        "flag": "--color",
                    }
                )
        elif flag == "--pretty":
            pretty = True
        elif flag == "--no-pretty":
            pretty = False
        elif flag == "--json":
            json_flag = True
        elif flag in ("-d", "--debug"):
            errors.append(
                {
                    "message": "No such option: --debug",
                    "failure": "invalid_flag",
                    "flag": "--debug",
                }
            )
        else:
            retained.append(flag)
    return GlobalCLIConfig(
        help=help_flag,
        quiet=quiet,
        verbose_level=verbose_level,
        fmt=fmt,
        pretty=pretty,
        log_level=log_level,
        color=color,
        json=json_flag,
        args=tuple(retained),
        errors=tuple(errors),
    )


__all__ = ["parse_global_flags"]
