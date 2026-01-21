# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Compatibility re-export for async runtime helpers."""

from __future__ import annotations

from bijux_cli.app.runtime import (  # noqa: F401
    AsyncTyper,
    adapt_typer,
    command_adapter,
    run_awaitable,
    run_command,
)
