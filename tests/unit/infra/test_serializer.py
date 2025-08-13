# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the infra serializer module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import builtins
import importlib
import io
import json
import sys
from types import SimpleNamespace
from typing import Any
from unittest.mock import ANY, MagicMock, patch

import pytest

from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import BijuxError
import bijux_cli.infra.serializer as infra_serializer
from bijux_cli.infra.serializer import OrjsonSerializer, PyYAMLSerializer, Redacted


class FakeStdout(io.StringIO):
    """A mock StringIO that can pretend to be a TTY."""

    def isatty(self) -> bool:
        """Simulate the isatty method."""
        return False


class FakeOrjson:
    """A mock of the orjson library for testing."""

    OPT_INDENT_2 = object()

    @staticmethod
    def dumps(obj: Any, option: int = 0, default: Any | None = None) -> bytes:
        """Simulate orjson.dumps, returning bytes."""
        return json.dumps(
            obj,
            indent=2 if option else None,
            ensure_ascii=False,
            default=default,
        ).encode("utf-8")

    @staticmethod
    def loads(data: bytes | bytearray | str) -> Any:
        """Simulate orjson.loads."""
        if isinstance(data, bytes | bytearray):
            data = data.decode()
        return json.loads(data)


def test_yaml_dump_success() -> None:
    """Test successful serialization to YAML."""
    out = infra_serializer.yaml_dump({"a": 1}, pretty=True)
    assert "a:" in out


def test_yaml_dump_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an error is raised when PyYAML is not available."""
    monkeypatch.setattr(infra_serializer, "_yaml_mod", None)
    with pytest.raises(BijuxError, match="PyYAML is required"):
        infra_serializer.yaml_dump({"a": 1}, pretty=False)


def test_redacted_str_and_to_json() -> None:
    """Test the behavior of the Redacted string subclass."""
    r = Redacted("secret")
    assert str(r) == "***"
    assert Redacted.to_json() == "***"
    assert isinstance(r, str)
    assert r == "secret"


def test_base_emit_writes_and_newline(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the base emit method writes JSON with a trailing newline."""
    tel = MagicMock()
    s = OrjsonSerializer(tel)
    fake = FakeStdout()
    monkeypatch.setattr(infra_serializer.sys, "stdout", fake)  # type: ignore[attr-defined]
    s.emit({"k": "v"}, fmt=OutputFormat.JSON, pretty=False)
    text = fake.getvalue().strip()
    assert text in ('{"k":"v"}', '{"k": "v"}')


def test_orjson_serializer_json_with_orjson(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test JSON serialization paths when orjson is available."""
    monkeypatch.setattr(infra_serializer, "_HAS_ORJSON", True)
    monkeypatch.setattr(infra_serializer, "_ORJSON", FakeOrjson)

    tel = MagicMock()
    s = OrjsonSerializer(tel)

    payload = {"a": 1, "secret": Redacted("top")}
    txt = s.dumps(payload, fmt=OutputFormat.JSON, pretty=True)
    assert '"a": 1' in txt
    assert '"secret": "top"' in txt
    tel.event.assert_any_call("serialize_dumps", {"format": "json", "pretty": True})

    raw = s.dumps_bytes(payload, fmt=OutputFormat.JSON, pretty=False)
    assert isinstance(raw, bytes | bytearray)
    tel.event.assert_any_call(
        "serialize_dumps_bytes", {"format": "json", "pretty": False}
    )

    back = s.loads(raw, fmt=OutputFormat.JSON)
    assert back["a"] == 1


def test_orjson_serializer_json_without_orjson_fallback(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test JSON serialization paths using the standard library fallback."""
    monkeypatch.setattr(infra_serializer, "_HAS_ORJSON", False)
    monkeypatch.setattr(infra_serializer, "_ORJSON", None)

    tel = MagicMock()
    s = OrjsonSerializer(tel)

    payload = {"a": 2, "secret": Redacted("xxx")}
    txt = s.dumps(payload, fmt=OutputFormat.JSON, pretty=True)
    assert '"a": 2' in txt
    assert '"secret": "xxx"' in txt
    tel.event.assert_any_call("serialize_dumps", {"format": "json", "pretty": True})

    raw = s.dumps_bytes(payload, fmt=OutputFormat.JSON, pretty=False)
    assert isinstance(raw, (bytes | bytearray))
    tel.event.assert_any_call(
        "serialize_dumps_bytes", {"format": "json", "pretty": False}
    )

    back = s.loads(txt, fmt=OutputFormat.JSON)
    assert back["a"] == 2


def test_orjson_serializer_json_default_typeerror_emits_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a TypeError during JSON serialization is handled."""
    monkeypatch.setattr(infra_serializer, "_HAS_ORJSON", False)
    monkeypatch.setattr(infra_serializer, "_ORJSON", None)

    tel = MagicMock()
    s = OrjsonSerializer(tel)

    class NotJSON:
        pass

    with pytest.raises(
        BijuxError, match=r"Failed to serialize json: .*JSON serialisable"
    ):
        s.dumps({"x": NotJSON()}, fmt=OutputFormat.JSON)

    tel.event.assert_any_call(
        "serialize_dumps_failed", {"format": "json", "error": ANY}
    )


def test_orjson_serializer_yaml_success() -> None:
    """Test successful YAML serialization and deserialization."""
    tel = MagicMock()
    s = OrjsonSerializer(tel)
    txt = s.dumps({"a": 3}, fmt=OutputFormat.YAML, pretty=True)
    assert "a:" in txt
    tel.event.assert_any_call("serialize_dumps", {"format": "yaml", "pretty": True})

    raw = s.dumps_bytes({"a": 3}, fmt=OutputFormat.YAML, pretty=False)
    assert isinstance(raw, (bytes | bytearray))
    tel.event.assert_any_call(
        "serialize_dumps_bytes", {"format": "yaml", "pretty": False}
    )

    back = s.loads("a: 3", fmt=OutputFormat.YAML)
    assert back["a"] == 3
    tel.event.assert_any_call("serialize_loads", {"format": "yaml"})


def test_orjson_serializer_yaml_missing(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that an error is raised if YAML is requested but not available."""
    tel = MagicMock()
    s = OrjsonSerializer(tel)

    monkeypatch.setattr(infra_serializer, "_HAS_YAML", False)
    monkeypatch.setattr(infra_serializer, "_YAML", None)

    with pytest.raises(BijuxError, match="PyYAML is required"):
        s.dumps({"a": 1}, fmt=OutputFormat.YAML)

    tel.event.assert_any_call(
        "serialize_dumps_failed", {"format": "yaml", "error": ANY}
    )

    with pytest.raises(BijuxError, match="PyYAML is required"):
        s.loads("a: 1", fmt=OutputFormat.YAML)

    tel.event.assert_any_call(
        "serialize_loads_failed", {"format": "yaml", "error": ANY}
    )


def test_orjson_serializer_unsupported_format() -> None:
    """Test that an unsupported serialization format raises an error."""
    tel = MagicMock()
    s = OrjsonSerializer(tel)

    class FakeFormat(SimpleNamespace):
        value = "fake"

    with pytest.raises(BijuxError, match="Unsupported format"):
        s.dumps({"a": 1}, fmt=FakeFormat)  # type: ignore[arg-type]

    with pytest.raises(BijuxError, match="Unsupported format"):
        s.dumps_bytes({"a": 1}, fmt=FakeFormat)  # type: ignore[arg-type]

    with pytest.raises(BijuxError, match="Unsupported format"):
        s.loads("{}", fmt=FakeFormat)  # type: ignore[arg-type]


def test_orjson_serializer_loads_json_with_orjson(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test JSON deserialization when orjson is available."""
    monkeypatch.setattr(infra_serializer, "_HAS_ORJSON", True)
    monkeypatch.setattr(infra_serializer, "_ORJSON", FakeOrjson)
    tel = MagicMock()
    s = OrjsonSerializer(tel)
    data = FakeOrjson.dumps({"k": "v"})
    out = s.loads(data, fmt=OutputFormat.JSON)
    assert out == {"k": "v"}
    tel.event.assert_any_call("serialize_loads", {"format": "json"})


def test_pyyaml_serializer_init_and_roundtrip() -> None:
    """Test the PyYAMLSerializer for a full serialization/deserialization roundtrip."""
    tel = MagicMock()
    y = PyYAMLSerializer(tel)
    txt = y.dumps({"k": Redacted("secret")}, fmt=OutputFormat.YAML, pretty=True)
    assert "***" in txt
    raw = y.dumps_bytes({"n": 1}, fmt=OutputFormat.YAML)
    assert isinstance(raw, (bytes | bytearray))
    back = y.loads("n: 1\n", fmt=OutputFormat.YAML)
    assert back == {"n": 1}


def test_pyyaml_serializer_unsupported() -> None:
    """Test that PyYAMLSerializer rejects non-YAML formats."""
    tel = MagicMock()
    y = PyYAMLSerializer(tel)

    with pytest.raises(BijuxError, match="only supports YAML"):
        y.dumps({"a": 1}, fmt=OutputFormat.JSON)

    with pytest.raises(BijuxError, match="only supports YAML"):
        y.loads("{}", fmt=OutputFormat.JSON)


def test_pyyaml_serializer_missing_yaml(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that PyYAMLSerializer raises an error if PyYAML is not installed."""
    monkeypatch.setattr(infra_serializer, "_HAS_YAML", False)
    monkeypatch.setattr(infra_serializer, "_YAML", None)
    with pytest.raises(BijuxError, match="PyYAML is not installed"):
        PyYAMLSerializer(MagicMock())


def test_serializer_for_json_and_yaml() -> None:
    """Test that the serializer_for factory returns the correct serializer class."""
    tel = MagicMock()
    s1 = infra_serializer.serializer_for("json", tel)
    assert isinstance(s1, OrjsonSerializer)
    s2 = infra_serializer.serializer_for(OutputFormat.YAML, tel)
    assert isinstance(s2, PyYAMLSerializer)


def test_import_fallback_branches(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the import logic for optional dependencies (orjson, yaml)."""
    modname = "bijux_cli.infra.serializer"
    if modname in sys.modules:
        sys.modules.pop(modname)
    real_import = builtins.__import__

    def fake_import(name: str, *args: Any, **kwargs: Any) -> Any:
        if name in ("orjson", "yaml"):
            raise ImportError("missing")
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", fake_import)
    s2 = importlib.import_module(modname)
    assert s2._ORJSON is None
    assert s2._YAML is None


def test_orjson_serializer_default_for_redacted() -> None:
    """Test the custom JSON default serializer for Redacted objects."""
    val = OrjsonSerializer._default(Redacted("foo"))
    assert val == "***" or val == "foo"


def test_orjson_serializer_default_typeerror() -> None:
    """Test that the default serializer raises a TypeError for unknown types."""

    class Dummy:
        pass

    with pytest.raises(TypeError):
        OrjsonSerializer._default(Dummy())


def test_emit_flushes_stdout_when_tty_true() -> None:
    """Test that stdout is flushed when emitting to a TTY."""
    tel = MagicMock()
    s = OrjsonSerializer(tel)

    class FakeTTY:
        def __init__(self) -> None:
            self.buf = ""
            self.flushed = False

        def write(self, s: str) -> None:
            self.buf += s

        def flush(self) -> None:
            self.flushed = True

        def isatty(self) -> bool:
            return True

    fake = FakeTTY()
    with patch("sys.stdout", fake):
        s.emit({"foo": "bar"}, fmt=OutputFormat.JSON)
    assert "\n" not in fake.buf
    assert fake.flushed
    assert json.loads(fake.buf) == {"foo": "bar"}
