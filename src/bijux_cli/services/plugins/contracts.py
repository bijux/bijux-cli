# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Service-level contracts for plugin configuration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class PluginConfig:
    """Configuration for plugin discovery and activation."""

    enabled: bool
    allow_entrypoints: bool


__all__ = ["PluginConfig"]
