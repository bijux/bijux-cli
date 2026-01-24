# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Formatting-only helpers for CLI output."""

from __future__ import annotations

from typing import Any

from bijux_cli.cli.core.emit import resolve_serializer
from bijux_cli.core.enums import OutputFormat


def format_payload(payload: Any, *, fmt: OutputFormat, pretty: bool) -> str:
    """Serialize a payload without applying policy decisions."""
    return resolve_serializer().dumps(payload, fmt=fmt, pretty=pretty)


__all__ = ["format_payload"]
