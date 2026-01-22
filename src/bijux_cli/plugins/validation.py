# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Plugin validation helpers shared by CLI and metadata."""

from __future__ import annotations

import re

PLUGIN_NAME_RE = re.compile(r"^[a-zA-Z0-9_-]+$")

__all__ = ["PLUGIN_NAME_RE"]
