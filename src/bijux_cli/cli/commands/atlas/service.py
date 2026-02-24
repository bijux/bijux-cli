# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Command surface for `bijux atlas ...`."""

from __future__ import annotations

import typer

from bijux_cli.cli.external_binaries import ExternalBinaryCommand, run_external
from bijux_cli.core.runtime import AsyncTyper

atlas_app = AsyncTyper(
    name="atlas",
    help="Bijux Atlas product runtime commands.",
    context_settings={
        "allow_extra_args": True,
        "ignore_unknown_options": True,
        "help_option_names": ["-h", "--help"],
    },
    add_help_option=True,
    no_args_is_help=False,
)


@atlas_app.callback(invoke_without_command=True)
def atlas(ctx: typer.Context) -> None:
    """Delegate to the `bijux-atlas` runtime binary."""

    if ctx.invoked_subcommand:
        return
    exit_code = run_external(
        ExternalBinaryCommand(
            bin_name="bijux-atlas",
            description="Bijux Atlas runtime",
            allowlist_key="atlas",
        ),
        list(ctx.args),
    )
    raise typer.Exit(exit_code)
