# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `dev list-products` command for product binary discovery."""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.external_binaries import probe_product_binaries, required_product_binaries
from bijux_cli.cli.core.command import new_run_command, validate_common_flags
from bijux_cli.cli.core.constants import OPT_FORMAT, OPT_LOG_LEVEL, OPT_PRETTY, OPT_QUIET
from bijux_cli.cli.core.help_text import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
)
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.core.version import __version__ as bijux_cli_version


def dev_list_products(
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """List product binaries and resolved paths."""

    command = "dev list-products"

    effective = current_execution_policy()
    validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )

    requirements = required_product_binaries()

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        payload: dict[str, object] = {
            "products": {
                name: [
                    {
                        "binary": entry.binary,
                        "path": entry.path,
                        "version": entry.version,
                        "compatible_major": entry.compatible_major,
                    }
                    for entry in probe_product_binaries(name, host_version=bijux_cli_version)
                ]
                for name in sorted(requirements.keys())
            }
        }
        if include_runtime:
            payload.update(
                {
                    "python": platform.python_version(),
                    "platform": platform.platform(),
                }
            )
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=effective.quiet,
        fmt=effective.output_format,
        pretty=effective.pretty,
        log_level=effective.log_level,
    )
