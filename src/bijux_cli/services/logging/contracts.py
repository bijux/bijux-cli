# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Service-level contracts for logging configuration."""

from __future__ import annotations

from dataclasses import dataclass

from bijux_cli.core.enums import ColorMode, LogLevel


@dataclass(frozen=True)
class LoggingConfig:
    """Configuration for logging and console output behavior."""

    quiet: bool
    verbose: bool
    log_level: LogLevel
    color: ColorMode


__all__ = ["LoggingConfig"]
