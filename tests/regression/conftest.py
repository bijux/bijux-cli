# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression test fixtures and markers."""

from __future__ import annotations

import pytest


@pytest.fixture(autouse=True)
def _register_real_serializer() -> None:
    """Override the basic serializer with the real implementation."""
    from bijux_cli.core.di import DIContainer
    from bijux_cli.infra.contracts import Serializer
    from bijux_cli.infra.serializer import OrjsonSerializer

    di = DIContainer.current()
    di.register(Serializer, lambda: OrjsonSerializer(None))
