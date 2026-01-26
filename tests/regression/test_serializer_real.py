# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression tests for real serializers."""

from __future__ import annotations

import pytest

from bijux_cli.core.enums import OutputFormat
from bijux_cli.infra.serializer import (
    OrjsonSerializer,
    PyYAMLSerializer,
    SerializationError,
)


def test_orjson_roundtrip_json() -> None:
    serializer = OrjsonSerializer(None)
    payload = {"alpha": 1, "beta": ["x", "y"]}
    dumped = serializer.dumps(payload, fmt=OutputFormat.JSON, pretty=False)
    loaded = serializer.loads(dumped, fmt=OutputFormat.JSON, pretty=False)
    assert loaded == payload


def test_pyyaml_roundtrip_yaml() -> None:
    serializer = PyYAMLSerializer(None)
    payload = {"alpha": 1, "beta": ["x", "y"]}
    dumped = serializer.dumps(payload, fmt=OutputFormat.YAML, pretty=True)
    loaded = serializer.loads(dumped, fmt=OutputFormat.YAML, pretty=False)
    assert loaded == payload


def test_emit_writes_to_stream() -> None:
    serializer = OrjsonSerializer(None)
    payload = {"alpha": 1}
    text = serializer.dumps(payload, fmt=OutputFormat.JSON, pretty=False)
    assert '"alpha":1' in text.replace(" ", "")


def test_invalid_format_raises() -> None:
    serializer = PyYAMLSerializer(None)
    with pytest.raises(SerializationError):
        serializer.dumps({"alpha": 1}, fmt=OutputFormat.JSON, pretty=False)
