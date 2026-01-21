# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Backward-compatible alias for core errors."""

from __future__ import annotations

from bijux_cli.core import errors as _errors
from bijux_cli.core.errors import *  # noqa: F403

__all__ = list(_errors.__all__)
