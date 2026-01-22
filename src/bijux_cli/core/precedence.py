# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Flag/env/config precedence helpers."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
import sys
from typing import Any


def parse_global_flags(
    argv: list[str],
    on_error: Callable[[str, str, dict[str, Any]], None],
) -> tuple[dict[str, Any], list[str]]:
    """Parse global CLI flags from argv and return flags + remaining args."""
    help_present = any(flag in ("-h", "--help") for flag in argv)
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
    for flag in it:
        if flag in ("-h", "--help"):
            flags["help"] = True
            retained.append(flag)
            continue
        if help_present:
            retained.append(flag.lstrip("-"))
            continue
        elif flag in ("-q", "--quiet"):
            flags["quiet"] = True
        elif flag in ("-d", "--debug"):
            flags["debug"] = True
            flags["verbose"] = True
        elif flag in ("-v", "--verbose"):
            flags["verbose"] = True
        elif flag in ("-f", "--format"):
            try:
                flags["format"] = next(it)
            except StopIteration:
                on_error("Missing value for --format.", "missing_argument", flags)
                break
            if flags["format"] not in ("json", "yaml"):
                on_error("Invalid output format.", "invalid_format", flags)
                break
        elif flag == "--log-level":
            try:
                flags["log_level"] = next(it)
            except StopIteration:
                on_error("Missing value for --log-level.", "missing_argument", flags)
                break
        elif flag == "--color":
            try:
                flags["color"] = next(it)
            except StopIteration:
                on_error("Missing value for --color.", "missing_argument", flags)
                break
            if flags["color"] not in ("auto", "always", "never"):
                on_error("Invalid color mode.", "invalid_color", flags)
                break
        elif flag == "--pretty":
            flags["pretty"] = True
        elif flag == "--no-pretty":
            flags["pretty"] = False
        else:
            retained.append(flag)
    return flags, retained


def apply_parsed_flags(flags: dict[str, Any], retained: list[str]) -> None:
    """Rewrite sys.argv with parsed global flags removed."""
    sys.argv = [sys.argv[0], *retained]


@dataclass(frozen=True)
class EffectiveConfig:
    """Resolved output/logging flags after precedence and normalization."""

    quiet: bool
    verbose_level: int
    debug: bool
    log_level: str
    color: str
    fmt: str
    pretty: bool
    include_runtime: bool
    json: bool


def resolve_effective_config(
    cli: dict[str, Any],
    env: dict[str, Any],
    file: dict[str, Any],
    defaults: dict[str, Any],
) -> EffectiveConfig:
    """Resolve flag/env/config precedence into a single effective config."""

    def _pick(key: str, fallback: Any) -> Any:
        for source in (cli, env, file, defaults):
            if key in source and source[key] is not None:
                return source[key]
            alt = key.replace("_", "-")
            if alt in source and source[alt] is not None:
                return source[alt]
        return fallback

    quiet = bool(_pick("quiet", False))
    debug = bool(_pick("debug", False))
    json_flag = bool(_pick("json", False))

    verbose_raw = _pick("verbose", 0)
    if isinstance(verbose_raw, bool):
        verbose_level = 1 if verbose_raw else 0
    elif isinstance(verbose_raw, int):
        verbose_level = max(0, verbose_raw)
    elif isinstance(verbose_raw, str) and verbose_raw.strip().isdigit():
        verbose_level = int(verbose_raw.strip())
    else:
        verbose_level = 0

    fmt = str(_pick("format", "json")).strip().lower()
    if json_flag:
        fmt = "json"

    pretty = bool(_pick("pretty", True))
    log_level = str(_pick("log_level", "info")).strip().lower()
    color = str(_pick("color", "auto")).strip().lower()
    if color not in ("auto", "always", "never"):
        color = "auto"

    if quiet:
        effective_log_level = "error"
    elif debug:
        effective_log_level = "debug"
    else:
        effective_log_level = log_level

    include_runtime = (verbose_level > 0 or debug) and not quiet
    effective_pretty = True if (debug and not quiet) else pretty
    return EffectiveConfig(
        quiet=quiet,
        verbose_level=verbose_level,
        debug=debug,
        log_level=effective_log_level,
        color=color,
        fmt=fmt,
        pretty=effective_pretty,
        include_runtime=include_runtime,
        json=json_flag,
    )


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
    effective = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "debug": debug,
            "pretty": pretty,
            "log_level": log_level,
            "color": color,
        },
        env={},
        file={},
        defaults={
            "quiet": False,
            "verbose": False,
            "debug": False,
            "pretty": True,
            "log_level": "info",
            "color": "auto",
            "format": "json",
            "json": False,
        },
    )
    return {
        "include_runtime": effective.include_runtime,
        "pretty": effective.pretty,
        "log_level": effective.log_level,
        "color": effective.color,
    }
