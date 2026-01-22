# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Flag/env/config precedence helpers."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class GlobalCLIConfig:
    """Immutable container for parsed global CLI flags."""

    help: bool
    quiet: bool
    verbose_level: int
    log_level: str
    fmt: str
    pretty: bool
    color: str
    json: bool
    args: tuple[str, ...]
    errors: tuple[dict[str, Any], ...]


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


@dataclass(frozen=True)
class ExecutionPolicy:
    """Resolved execution policy shared across CLI/service boundaries."""

    output_format: str
    color: str
    quiet: bool
    verbose: bool
    verbose_level: int
    log_level: str
    pretty: bool
    include_runtime: bool
    json: bool

    @property
    def fmt(self) -> str:
        """Backward-compatible alias for output format."""
        return self.output_format


def _coerce_verbose(value: Any) -> int:
    if isinstance(value, bool):
        return 1 if value else 0
    if isinstance(value, int):
        return max(0, value)
    if isinstance(value, str) and value.strip().isdigit():
        return int(value.strip())
    return 0


def _normalize_str(value: Any, default: str) -> str:
    return str(value or default).strip().lower()


def _normalize_bool(value: Any, default: bool) -> bool:
    if value is None:
        return default
    return bool(value)


def _cli_to_dict(cli: GlobalCLIConfig | Mapping[str, Any]) -> dict[str, Any]:
    if isinstance(cli, GlobalCLIConfig):
        return {
            "help": cli.help,
            "quiet": cli.quiet,
            "verbose": cli.verbose_level,
            "format": cli.fmt,
            "pretty": cli.pretty,
            "log_level": cli.log_level,
            "color": cli.color,
            "json": cli.json,
        }
    return dict(cli)


def validate_cli_flags(
    config: GlobalCLIConfig, parse_errors: Sequence[dict[str, Any]] | None = None
) -> list[dict[str, Any]]:
    """Validate raw CLI flags without applying behavior."""
    errors: list[dict[str, Any]] = list(parse_errors or config.errors)
    fmt = _normalize_str(config.fmt, "")
    if fmt and fmt not in ("json", "yaml"):
        errors.append(
            {
                "message": "Invalid output format.",
                "failure": "invalid_format",
                "flag": "--format",
            }
        )
    color = _normalize_str(config.color, "")
    if color and color not in ("auto", "always", "never"):
        errors.append(
            {
                "message": "Invalid color mode.",
                "failure": "invalid_color",
                "flag": "--color",
            }
        )
    log_level = _normalize_str(config.log_level, "info")
    if log_level and log_level not in ("debug", "info", "warning", "error", "critical"):
        errors.append(
            {
                "message": "Invalid log level.",
                "failure": "invalid_log_level",
                "flag": "--log-level",
            }
        )
    return errors


def _pick_value(key: str, sources: Sequence[Mapping[str, Any]], fallback: Any) -> Any:
    for source in sources:
        if key in source and source[key] is not None:
            return source[key]
        alt = key.replace("_", "-")
        if alt in source and source[alt] is not None:
            return source[alt]
    return fallback


def _resolve_base(
    cli: Mapping[str, Any],
    env: Mapping[str, Any],
    file: Mapping[str, Any],
    defaults: Mapping[str, Any],
) -> dict[str, Any]:
    sources = (cli, env, file, defaults)
    quiet = _normalize_bool(_pick_value("quiet", sources, False), False)
    json_flag = _normalize_bool(_pick_value("json", sources, False), False)
    verbose_level = _coerce_verbose(_pick_value("verbose", sources, 0))
    fmt = _normalize_str(_pick_value("format", sources, "json"), "json")
    if json_flag:
        fmt = "json"
    pretty = _normalize_bool(_pick_value("pretty", sources, True), True)
    log_level = _normalize_str(_pick_value("log_level", sources, "info"), "info")
    color = _normalize_str(_pick_value("color", sources, "auto"), "auto")
    return {
        "quiet": quiet,
        "json": json_flag,
        "verbose_level": verbose_level,
        "fmt": fmt,
        "pretty": pretty,
        "log_level": log_level,
        "color": color,
    }


def _normalize_effective(base: Mapping[str, Any]) -> EffectiveConfig:
    quiet = bool(base["quiet"])
    log_level = str(base["log_level"])
    color = str(base["color"])
    if color not in ("auto", "always", "never"):
        color = "auto"
    effective_log_level = "error" if quiet else log_level
    verbose_level = int(base["verbose_level"])
    include_runtime = (
        verbose_level > 0 or effective_log_level == "debug"
    ) and not quiet
    pretty = bool(base["pretty"])
    effective_pretty = (
        True if (effective_log_level == "debug" and not quiet) else pretty
    )
    return EffectiveConfig(
        quiet=quiet,
        verbose_level=verbose_level,
        log_level=effective_log_level,
        color=color,
        fmt=str(base["fmt"]),
        pretty=effective_pretty,
        include_runtime=include_runtime,
        json=bool(base["json"]),
    )


def resolve_effective_config(
    cli: GlobalCLIConfig | Mapping[str, Any],
    env: Mapping[str, Any],
    file: Mapping[str, Any],
    defaults: Mapping[str, Any],
) -> EffectiveConfig:
    """Resolve flag/env/config precedence into a single effective config."""
    base = _resolve_base(_cli_to_dict(cli), env, file, defaults)
    return _normalize_effective(base)


def resolve_execution_policy(effective: EffectiveConfig) -> ExecutionPolicy:
    """Create an immutable execution policy from resolved config."""
    return ExecutionPolicy(
        output_format=effective.fmt,
        color=effective.color,
        quiet=effective.quiet,
        verbose=effective.verbose_level > 0,
        verbose_level=effective.verbose_level,
        log_level=effective.log_level,
        pretty=effective.pretty,
        include_runtime=effective.include_runtime,
        json=effective.json,
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
