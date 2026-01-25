# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra serializer module."""

from __future__ import annotations

import json
from typing import cast
from unittest.mock import MagicMock

import pytest

from bijux_cli.core.enums import OutputFormat
from bijux_cli.infra.serializer import (
    OrjsonSerializer,
    SerializationError,
    serializer_for,
)
from bijux_cli.services.contracts import TelemetryProtocol


def test_orjson_serializer_json_roundtrip() -> None:
    """OrjsonSerializer should serialize JSON by default."""
    tel = MagicMock(spec=TelemetryProtocol)
    serializer = OrjsonSerializer(tel)
    payload = {"a": 1}
    dumped = serializer.dumps(payload)
    assert json.loads(dumped) == payload


def test_serializer_for_rejects_unknown_format() -> None:
    """serializer_for should reject unknown formats."""
    tel = MagicMock(spec=TelemetryProtocol)
    with pytest.raises(SerializationError, match="Unsupported format"):
        serializer_for(cast(OutputFormat, "toml"), tel)
