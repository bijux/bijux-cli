# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra serializer module."""

from __future__ import annotations

from collections.abc import Callable
import json
from typing import Any, cast
from unittest.mock import MagicMock

import pytest

from bijux_cli.core.enums import OutputFormat
import bijux_cli.infra.serializer as serializer_mod
from bijux_cli.infra.serializer import (
    OrjsonSerializer,
    PyYAMLSerializer,
    serializer_for,
)
from bijux_cli.services.contracts import TelemetryProtocol


def test_orjson_serializer_json_roundtrip() -> None:
    """OrjsonSerializer should serialize JSON by default."""
    tel = MagicMock(spec=TelemetryProtocol)
    serializer = OrjsonSerializer(tel)
    payload = {"a": 1}
    dumped = serializer.dumps(payload, fmt=OutputFormat.JSON, pretty=False)
    assert json.loads(dumped) == payload


def test_orjson_serializer_bytes_roundtrip() -> None:
    tel = MagicMock(spec=TelemetryProtocol)
    serializer = OrjsonSerializer(tel)
    payload = {"a": 1}
    dumped = serializer.dumps_bytes(payload, fmt=OutputFormat.JSON, pretty=False)
    assert json.loads(dumped.decode("utf-8")) == payload


def test_pyyaml_serializer_rejects_json() -> None:
    tel = MagicMock(spec=TelemetryProtocol)
    serializer = PyYAMLSerializer(tel)
    with pytest.raises(serializer_mod.SerializationError):
        serializer.dumps({"a": 1}, fmt=OutputFormat.JSON, pretty=False)


def test_serializer_for_yaml_roundtrip() -> None:
    tel = MagicMock(spec=TelemetryProtocol)
    serializer = serializer_for(OutputFormat.YAML, tel)
    dumped = serializer.dumps({"a": 1}, fmt=OutputFormat.YAML, pretty=True)
    loaded = serializer.loads(dumped, fmt=OutputFormat.YAML, pretty=False)
    assert loaded == {"a": 1}


def test_yaml_dump_requires_pyyaml(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(serializer_mod, "_YAML", None)
    serializer = serializer_mod.OrjsonSerializer(None)
    with pytest.raises(serializer_mod.SerializationError):
        serializer.dumps({"a": 1}, fmt=OutputFormat.YAML, pretty=False)


def test_serializer_import_without_orjson(monkeypatch: pytest.MonkeyPatch) -> None:
    import builtins
    import importlib

    import bijux_cli.infra.serializer as serializer_mod

    real_import = cast(Callable[..., object], builtins.__import__)

    def fake_import(name: str, *args: object, **kwargs: object) -> object:
        if name == "orjson":
            raise ImportError
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", fake_import)
    importlib.reload(serializer_mod)
    assert serializer_mod._ORJSON is None
    importlib.reload(serializer_mod)


def test_serializer_import_without_yaml(monkeypatch: pytest.MonkeyPatch) -> None:
    import builtins
    import importlib

    import bijux_cli.infra.serializer as serializer_mod

    real_import = cast(Callable[..., object], builtins.__import__)

    def fake_import(name: str, *args: object, **kwargs: object) -> object:
        if name == "yaml":
            raise ImportError
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", fake_import)
    importlib.reload(serializer_mod)
    assert serializer_mod._YAML is None
    importlib.reload(serializer_mod)


def test_serializer_for_rejects_unknown_format() -> None:
    """serializer_for should reject unknown formats."""
    tel = MagicMock(spec=TelemetryProtocol)
    with pytest.raises(serializer_mod.SerializationError, match="Unsupported format"):
        serializer_mod.serializer_for(cast(OutputFormat, "toml"), tel)


def test_orjson_serializer_falls_back_to_stdlib(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    import bijux_cli.infra.serializer as serializer_mod

    monkeypatch.setattr(serializer_mod, "_ORJSON", None)
    serializer = serializer_mod.OrjsonSerializer(None)
    dumped = serializer.dumps({"a": 1}, fmt=OutputFormat.JSON, pretty=True)
    assert json.loads(dumped) == {"a": 1}


def test_orjson_serializer_json_errors(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(serializer_mod, "_ORJSON", None)
    serializer = serializer_mod.OrjsonSerializer(None)
    with pytest.raises(
        serializer_mod.SerializationError, match="Failed to serialize json"
    ):
        serializer.dumps({"a": set()}, fmt=OutputFormat.JSON, pretty=False)


def test_orjson_serializer_loads_errors() -> None:
    serializer = OrjsonSerializer(None)
    with pytest.raises(
        serializer_mod.SerializationError, match="Failed to deserialize json"
    ):
        serializer.loads("{bad json}", fmt=OutputFormat.JSON, pretty=False)


def test_orjson_serializer_unsupported_format() -> None:
    serializer = OrjsonSerializer(None)
    with pytest.raises(serializer_mod.SerializationError, match="Unsupported format"):
        serializer.dumps({}, fmt=cast(OutputFormat, "xml"), pretty=False)


def test_orjson_serializer_yaml_loads_requires_yaml(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(serializer_mod, "_YAML", None)
    serializer = OrjsonSerializer(None)
    with pytest.raises(serializer_mod.SerializationError, match="PyYAML is required"):
        serializer.loads("{}", fmt=OutputFormat.YAML, pretty=False)


def test_orjson_serializer_yaml_loads_with_yaml(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class _Y:
        @staticmethod
        def safe_load(_data: object) -> dict[str, int]:
            return {"a": 1}

    monkeypatch.setattr(serializer_mod, "_YAML", _Y)
    serializer = OrjsonSerializer(None)
    assert serializer.loads("a: 1", fmt=OutputFormat.YAML, pretty=False) == {"a": 1}


def test_orjson_serializer_unsupported_load_format() -> None:
    serializer = OrjsonSerializer(None)
    with pytest.raises(serializer_mod.SerializationError, match="Unsupported format"):
        serializer.loads("{}", fmt=cast(OutputFormat, "xml"), pretty=False)


def test_pyyaml_serializer_invalid_format() -> None:
    class _Y:
        @staticmethod
        def safe_dump(_obj: object, **_kwargs: object) -> str:
            return "a: 1"

        @staticmethod
        def safe_load(_data: object) -> dict[str, int]:
            return {"a": 1}

    ser_any = cast(Any, serializer_mod)
    ser_any._YAML = _Y
    serializer = PyYAMLSerializer(None)
    with pytest.raises(serializer_mod.SerializationError, match="only supports YAML"):
        serializer.loads("{}", fmt=OutputFormat.JSON, pretty=False)


def test_pyyaml_serializer_requires_yaml(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(serializer_mod, "_YAML", None)
    with pytest.raises(
        serializer_mod.SerializationError, match="PyYAML is not installed"
    ):
        serializer_mod.PyYAMLSerializer(None)


def test_pyyaml_serializer_loads_when_yaml_missing(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class _Y:
        @staticmethod
        def safe_dump(_obj: object, **_kwargs: object) -> str:
            return "a: 1"

        @staticmethod
        def safe_load(_data: object) -> dict[str, int]:
            return {"a": 1}

    monkeypatch.setattr(serializer_mod, "_YAML", _Y)
    serializer = serializer_mod.PyYAMLSerializer(None)
    monkeypatch.setattr(serializer_mod, "_YAML", None)
    assert serializer.loads("x: 1", fmt=OutputFormat.YAML, pretty=False) is None


def test_pyyaml_serializer_dumps_bytes() -> None:
    class _Y:
        @staticmethod
        def safe_dump(_obj: object, **_kwargs: object) -> str:
            return "a: 1"

        @staticmethod
        def safe_load(_data: object) -> dict[str, int]:
            return {"a": 1}

    ser_any = cast(Any, serializer_mod)
    ser_any._YAML = _Y
    serializer = serializer_mod.PyYAMLSerializer(None)
    dumped = serializer.dumps_bytes({"a": 1}, fmt=OutputFormat.YAML, pretty=True)
    assert isinstance(dumped, bytes)


def test_serializer_for_json_returns_orjson() -> None:
    serializer = serializer_for(OutputFormat.JSON, None)
    assert isinstance(serializer, serializer_mod.OrjsonSerializer)
