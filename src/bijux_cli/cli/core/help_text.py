# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Help text strings for CLI commands."""

from __future__ import annotations

HELP_QUIET = "Suppress normal output; exit code still indicates success/failure."
HELP_NO_PRETTY = "Disable pretty-printing (indentation) in JSON/YAML output."
HELP_FORMAT = "Machine-readable output format (json|yaml); defaults to json."
HELP_LOG_LEVEL = "Set logging level (trace|debug|info). Example: --log-level debug."
HELP_FORMAT_HELP = "Output format: human (default), json, yaml."

__all__ = [
    "HELP_QUIET",
    "HELP_NO_PRETTY",
    "HELP_FORMAT",
    "HELP_LOG_LEVEL",
    "HELP_FORMAT_HELP",
]
