# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Command surface for `bijux dev atlas ...`."""

from __future__ import annotations

import typer

from bijux_cli.cli.external_binaries import ExternalBinaryCommand, run_external
from bijux_cli.core.runtime import AsyncTyper

dev_atlas_app = AsyncTyper(
    name="atlas",
    help="Bijux Atlas control-plane developer commands.",
    context_settings={
        "allow_extra_args": True,
        "ignore_unknown_options": True,
        "help_option_names": ["-h", "--help"],
    },
    add_help_option=True,
    no_args_is_help=False,
)


@dev_atlas_app.callback(invoke_without_command=True)
def dev_atlas(ctx: typer.Context) -> None:
    """Delegate to the `bijux-dev-atlas` control-plane binary."""

    if ctx.invoked_subcommand:
        return
    exit_code = run_external(
        ExternalBinaryCommand(
            bin_name="bijux-dev-atlas",
            description="Bijux Atlas control-plane",
            allowlist_key="dev-atlas",
        ),
        list(ctx.args),
    )
    raise typer.Exit(exit_code)
