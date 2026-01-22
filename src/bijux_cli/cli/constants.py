# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines shared constants and help text for CLI commands."""

from __future__ import annotations

HELP_VERBOSE = "Include extra runtime details."
HELP_QUIET = "Suppress normal output; exit code still indicates success/failure."
HELP_NO_PRETTY = "Disable pretty-printing (indentation) in JSON/YAML output."
HELP_FORMAT = "Machine-readable output format (json|yaml); defaults to json."
HELP_LOG_LEVEL = "Set logging level (debug|info|warning|error|critical)."
HELP_FORMAT_HELP = "Output format: human (default), json, yaml."

DEFAULT_COMMAND_TIMEOUT = 30.0
