# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Service-level contracts for logging configuration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class LoggingConfig:
    """Configuration for logging and console output behavior."""

    debug: bool
    quiet: bool
    verbose: bool
    log_level: str
    color: str


__all__ = ["LoggingConfig"]
