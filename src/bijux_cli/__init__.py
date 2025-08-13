# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""The top-level package for the Bijux CLI application.

This module serves as the main public entry point for the `bijux-cli` package.
It exposes the key components required for both command-line execution and
programmatic integration.

The primary exports are:
    * `entry_point`: The function used by `console_scripts` to start the CLI.
    * `BijuxAPI`: A high-level, synchronous facade for using the CLI's
        functionality within other Python applications.
    * `version` and `api_version`: The application and plugin API versions.
"""

from __future__ import annotations

from bijux_cli.__main__ import main
from bijux_cli.__version__ import api_version, version
from bijux_cli.api import BijuxAPI


def entry_point() -> int | None:
    """The primary entry point for the `console_scripts` definition.

    This function calls the main CLI orchestrator and catches `SystemExit`
    exceptions to ensure a proper integer exit code is returned to the shell.

    Returns:
        int | None: The integer exit code of the CLI process.
    """
    try:
        return main()
    except SystemExit as exc:
        return int(exc.code or 0)


__all__ = ["version", "api_version", "BijuxAPI", "entry_point"]
