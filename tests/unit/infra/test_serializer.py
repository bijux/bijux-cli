# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra serializer module."""

from __future__ import annotations

import json
from unittest.mock import MagicMock

import pytest

from bijux_cli.core.enums import OutputFormat
from bijux_cli.infra.serializer import (
    OrjsonSerializer,
    PyYAMLSerializer,
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


def test_orjson_serializer_yaml_requires_yaml() -> None:
    """YAML serialization should raise when PyYAML is missing."""
    from bijux_cli.infra import serializer as serializer_mod

    if serializer_mod._YAML is not None:
        pytest.skip("PyYAML is available")
    tel = MagicMock(spec=TelemetryProtocol)
    serializer = OrjsonSerializer(tel)
    with pytest.raises(SerializationError, match="PyYAML is required"):
        _ = serializer.dumps({"a": 1}, fmt=OutputFormat.YAML)


def test_serializer_for_rejects_unknown_format() -> None:
    """serializer_for should reject unknown formats."""
    tel = MagicMock(spec=TelemetryProtocol)
    with pytest.raises(SerializationError, match="Unsupported format"):
        serializer_for("toml", tel)


def test_pyyaml_serializer_requires_yaml() -> None:
    """PyYAMLSerializer should raise if PyYAML is unavailable."""
    from bijux_cli.infra import serializer as serializer_mod

    if serializer_mod._YAML is not None:
        pytest.skip("PyYAML is available")
    tel = MagicMock(spec=TelemetryProtocol)
    with pytest.raises(SerializationError, match="PyYAML is not installed"):
        _ = PyYAMLSerializer(tel)
