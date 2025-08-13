# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines shared constants and help text for the Bijux CLI.

This module centralizes constant values used throughout the application,
such as default timeouts and standardized help messages for common CLI flags.
This practice avoids magic strings and numbers, improving maintainability and
ensuring consistency across all commands.
"""

from __future__ import annotations

HELP_DEBUG = "Enable diagnostics to stderr."
HELP_VERBOSE = "Include extra runtime details."
HELP_QUIET = "Suppress normal output; exit code still indicates success/failure."
HELP_NO_PRETTY = "Disable pretty‑printing (indentation) in JSON/YAML output."
HELP_FORMAT = "Machine‑readable output format (json|yaml); defaults to json."
HELP_FORMAT_HELP = "Output format: human (default), json, yaml."

DEFAULT_COMMAND_TIMEOUT = 30.0
