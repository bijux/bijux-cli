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
from bijux_cli.infra.telemetry import Telemetry


def test_orjson_serializer_json_roundtrip() -> None:
    """OrjsonSerializer should serialize JSON by default."""
    tel = MagicMock(spec=Telemetry)
    ser = OrjsonSerializer(tel)
    payload = {"a": 1}
    dumped = ser.dumps(payload)
    assert json.loads(dumped) == payload


def test_orjson_serializer_yaml_requires_yaml() -> None:
    """YAML serialization should raise when PyYAML is missing."""
    tel = MagicMock(spec=Telemetry)
    ser = OrjsonSerializer(tel)
    try:
        _ = ser.dumps({"a": 1}, fmt=OutputFormat.YAML)
    except SerializationError as exc:
        assert "PyYAML is required" in str(exc)


def test_serializer_for_rejects_unknown_format() -> None:
    """serializer_for should reject unknown formats."""
    tel = MagicMock(spec=Telemetry)
    with pytest.raises(SerializationError, match="Unsupported format"):
        serializer_for("toml", tel)


def test_pyyaml_serializer_requires_yaml() -> None:
    """PyYAMLSerializer should raise if PyYAML is unavailable."""
    tel = MagicMock(spec=Telemetry)
    try:
        _ = PyYAMLSerializer(tel)
    except SerializationError as exc:
        assert "PyYAML is not installed" in str(exc)
