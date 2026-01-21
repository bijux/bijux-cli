# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's infrastructure layer.

The infra package is intentionally minimal: only OS/IO utilities live here.
Service implementations that depend on core protocols or errors are housed
under `services/` instead of `infra/`.
"""

from __future__ import annotations

from bijux_cli.infra.fs import *  # noqa: F403
from bijux_cli.infra.terminal import *  # noqa: F403

__all__: list[str] = []
