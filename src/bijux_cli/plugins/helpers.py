# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Shared plugin helpers (validation, listing, cache)."""

from __future__ import annotations

import re
from typing import Any

PLUGIN_NAME_RE = re.compile(r"^[a-zA-Z0-9_-]+$")


def list_installed_plugins() -> list[dict[str, Any]]:
    """Return installed plugin metadata."""
    from bijux_cli.plugins.metadata import list_plugins

    return list_plugins()


def invalidate_cache() -> None:
    """Invalidate the plugin discovery cache."""
    from bijux_cli.plugins.metadata import invalidate_plugin_cache

    invalidate_plugin_cache()


__all__ = ["PLUGIN_NAME_RE", "list_installed_plugins", "invalidate_cache"]
