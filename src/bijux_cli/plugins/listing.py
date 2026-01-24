# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin listing helpers."""

from __future__ import annotations

from typing import Any

from bijux_cli.plugins.metadata import list_plugins


def list_installed_plugins() -> list[dict[str, Any]]:
    """Return installed plugin metadata."""
    return list_plugins()


__all__ = ["list_installed_plugins"]
