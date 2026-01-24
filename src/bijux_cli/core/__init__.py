# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's core components.

This module acts as the public facade for the most essential classes and
exceptions defined within the `bijux_cli.core` package. It aggregates these
components into a single, stable namespace for convenient importing throughout
the application.

By exposing these fundamental building blocks here, the rest of the application
can remain decoupled from the internal structure of the `core` package.

The exposed components include:
* **Enumerations:** `OutputFormat`.
* **Custom Exceptions:** `BijuxError`, `ConfigError`, `PluginError`, etc.
"""

from __future__ import annotations

from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.errors import (
    BijuxError,
    ConfigError,
    InternalError,
    PluginError,
    UserInputError,
)

__all__ = [
    "OutputFormat",
    "BijuxError",
    "ConfigError",
    "InternalError",
    "PluginError",
    "UserInputError",
]
