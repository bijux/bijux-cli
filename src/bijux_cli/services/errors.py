# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Service-layer error types for Bijux CLI."""

from __future__ import annotations

from bijux_cli.core.errors import InternalError


class ServiceError(InternalError):
    """Raised for service-related failures."""


__all__ = ["ServiceError"]
