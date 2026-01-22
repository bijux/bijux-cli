# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Flag/env/config precedence helpers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class EffectiveConfig:
    """Resolved output/logging flags after precedence and normalization."""

    quiet: bool
    verbose_level: int
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

    effective_log_level = "error" if quiet else log_level

    include_runtime = (
        verbose_level > 0 or effective_log_level == "debug"
    ) and not quiet
    effective_pretty = (
        True if (effective_log_level == "debug" and not quiet) else pretty
    )
    return EffectiveConfig(
        quiet=quiet,
        verbose_level=verbose_level,
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
    pretty: bool,
    log_level: str = "info",
    color: str = "auto",
) -> dict[str, Any]:
    """Resolve logging/color/pretty flags from a single source of truth."""
    effective = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "pretty": pretty,
            "log_level": log_level,
            "color": color,
        },
        env={},
        file={},
        defaults={
            "quiet": False,
            "verbose": False,
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
