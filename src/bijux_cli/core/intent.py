# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Pure CLI parsing and intent construction."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
import os

from bijux_cli.cli.color import resolve_color_mode
from bijux_cli.cli.core.constants import (
    ENV_COLOR,
    ENV_LOG_LEVEL,
    ENV_NO_COLOR,
    OPT_HELP,
)
from bijux_cli.cli.core.flags import collect_global_flag_errors, parse_global_flags
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import (
    EffectiveConfig,
    FlagError,
    FlagLayer,
    Flags,
    GlobalCLIConfig,
    LogPolicy,
    resolve_effective_config,
    resolve_log_policy,
    validate_cli_flags,
)


@dataclass(frozen=True)
class CLIIntent:
    """Resolved, side-effect-free CLI intent."""

    command: str | None
    args: tuple[str, ...]
    flags: Flags
    output_format: OutputFormat
    log_level: LogLevel
    quiet: bool
    color: ColorMode
    pretty: bool
    include_runtime: bool
    log_policy: LogPolicy
    help: bool
    errors: tuple[FlagError, ...]


def parse_global_config(argv: list[str]) -> GlobalCLIConfig:
    """Parse global CLI flags once at the CLI root layer."""
    flags = parse_global_flags(argv)
    help_flag = any(arg in OPT_HELP for arg in argv)
    errors = () if help_flag else collect_global_flag_errors(argv)
    return GlobalCLIConfig(
        help=help_flag,
        flags=FlagLayer(
            quiet=flags.quiet,
            log_level=flags.log_level,
            color=flags.color,
            format=flags.format,
        ),
        args=tuple(argv),
        errors=errors,
    )


def split_command_args(args: list[str]) -> tuple[str | None, list[str]]:
    """Return the first command token and remaining args."""
    from bijux_cli.cli.core.constants import (
        OPT_COLOR,
        OPT_FORMAT,
        OPT_LOG_LEVEL,
        OPT_QUIET,
        PRETTY_FLAGS,
    )

    flags_with_values = {*OPT_FORMAT, *OPT_LOG_LEVEL, *OPT_COLOR}
    flags_no_values = {*OPT_QUIET, *PRETTY_FLAGS, *OPT_HELP}
    i = 0
    while i < len(args):
        arg = args[i]
        if arg in flags_with_values:
            i += 2
            continue
        if arg in flags_no_values or arg.startswith("-"):
            i += 1
            continue
        return arg, args[i + 1 :]
    return None, []


def build_cli_intent(
    args: list[str],
    *,
    env: Mapping[str, str] | None = None,
    tty: bool = True,
) -> CLIIntent:
    """Build an intent from CLI args and supplied environment."""
    env = env or os.environ
    parsed = parse_global_config(args)
    errors = validate_cli_flags(parsed)

    env_log = env.get(ENV_LOG_LEVEL)
    env_color = env.get(ENV_COLOR)
    effective = resolve_effective_config(
        cli=parsed.flags,
        env=FlagLayer(
            log_level=LogLevel(env_log) if env_log else None,
            color=ColorMode(env_color) if env_color else None,
        ),
        file=FlagLayer(),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
    )

    color_config = GlobalCLIConfig(
        help=parsed.help,
        flags=FlagLayer(color=effective.flags.color),
        args=parsed.args,
        errors=parsed.errors,
    )
    resolved_color = resolve_color_mode(
        color_config,
        tty,
        no_color=env.get(ENV_NO_COLOR) == "1",
    )
    if resolved_color != effective.flags.color:
        effective = EffectiveConfig(
            flags=Flags(
                quiet=effective.flags.quiet,
                log_level=effective.flags.log_level,
                color=resolved_color,
                format=effective.flags.format,
            )
        )

    log_policy = resolve_log_policy(effective.flags.log_level)
    command, _ = split_command_args(args)
    return CLIIntent(
        command=command,
        args=tuple(args),
        flags=effective.flags,
        output_format=effective.flags.format,
        log_level=effective.flags.log_level,
        quiet=effective.flags.quiet,
        color=effective.flags.color,
        pretty=log_policy.pretty_default,
        include_runtime=log_policy.show_internal,
        log_policy=log_policy,
        help=parsed.help,
        errors=tuple(errors),
    )


def current_cli_intent() -> CLIIntent:
    """Resolve the current CLI intent from DI or fallback to argv/env."""
    import sys

    from bijux_cli.core.di import DIContainer

    try:
        intent: object = DIContainer.current().resolve(CLIIntent)
    except Exception:
        intent = None
    if isinstance(intent, CLIIntent):
        return intent
    args = [] if os.environ.get("PYTEST_CURRENT_TEST") else list(sys.argv[1:])
    return build_cli_intent(
        args,
        env=os.environ,
        tty=sys.stdout.isatty(),
    )


__all__ = [
    "CLIIntent",
    "build_cli_intent",
    "current_cli_intent",
    "parse_global_config",
    "split_command_args",
]
