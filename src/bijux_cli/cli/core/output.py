# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Formatting-only helpers for CLI output."""

from __future__ import annotations

from typing import Any

from bijux_cli.cli.core.emit import resolve_serializer
from bijux_cli.core.precedence import OutputConfig


def format_payload(payload: Any, *, output: OutputConfig) -> str:
    """Serialize a payload using a resolved output config."""
    return resolve_serializer().dumps(payload, fmt=output.format, pretty=output.pretty)


__all__ = ["format_payload"]
