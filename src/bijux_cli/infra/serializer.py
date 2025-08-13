# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides concrete serialization services for JSON and YAML formats.

This module defines concrete implementations of the `SerializerProtocol`. It
offers different serializers optimized for performance and specific formats,
gracefully handling optional dependencies.

Key components include:
    * `OrjsonSerializer`: A high-performance serializer that uses `orjson` for
        JSON serialization if installed, falling back to the standard `json`
        module. It uses `PyYAML` for YAML.
    * `PyYAMLSerializer`: A serializer that exclusively handles the YAML format.
    * `Redacted`: A special string subclass for wrapping sensitive data to prevent
        it from being exposed in serialized output.
    * `serializer_for`: A factory function that returns the most appropriate
        serializer instance for a given format.
"""

from __future__ import annotations

import abc
import importlib.util as _importlib_util
import json
import sys
from types import ModuleType
import typing
from typing import TYPE_CHECKING, Any, Final

from injector import inject

from bijux_cli.contracts import TelemetryProtocol
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import BijuxError

_orjson_spec = _importlib_util.find_spec("orjson")
_yaml_spec = _importlib_util.find_spec("yaml")

_orjson_mod: ModuleType | None
try:
    import orjson as _orjson_mod
except ImportError:
    _orjson_mod = None
_ORJSON: Final[ModuleType | None] = _orjson_mod

_yaml_mod: ModuleType | None
try:
    import yaml as _yaml_mod  # pyright: ignore[reportMissingModuleSource]
except ImportError:
    _yaml_mod = None
_YAML: Final[ModuleType | None] = _yaml_mod

_HAS_ORJSON: Final[bool] = _ORJSON is not None
_HAS_YAML: Final[bool] = _YAML is not None


def yaml_dump(obj: Any, pretty: bool) -> str:
    """Dumps an object to a YAML string using PyYAML.

    Args:
        obj (Any): The object to serialize.
        pretty (bool): If True, formats the output in an indented block style.

    Returns:
        str: The serialized YAML string.

    Raises:
        BijuxError: If the `PyYAML` library is not installed.
    """
    if _yaml_mod is None:
        raise BijuxError("PyYAML is required for YAML operations")
    dumped = _yaml_mod.safe_dump(
        obj,
        sort_keys=False,
        default_flow_style=not pretty,
        indent=2 if pretty else None,
    )
    return dumped or ""


class Redacted(str):
    """A string subclass that redacts its value when printed or serialized.

    This is used to wrap sensitive data, such as secrets or API keys, to
    prevent them from being accidentally exposed in logs or console output.
    """

    def __new__(cls, value: str) -> Redacted:
        """Creates a new `Redacted` string instance.

        Args:
            value (str): The original, sensitive value to be wrapped.

        Returns:
            Redacted: The new `Redacted` string instance.
        """
        return str.__new__(cls, value)

    def __str__(self) -> str:
        """Returns the redacted representation.

        Returns:
            str: A static string "***" to represent the redacted value.
        """
        return "***"

    @staticmethod
    def to_json() -> str:
        """Provides a JSON-serializable representation for libraries like `orjson`.

        Returns:
            str: A static string "***" to represent the redacted value.
        """
        return "***"


class _Base(abc.ABC):
    """An abstract base class for all serializer implementations.

    Attributes:
        _telemetry (TelemetryProtocol | None): The telemetry service for events.
    """

    def __init__(self, telemetry: TelemetryProtocol | None) -> None:
        """Initializes the base serializer.

        Args:
            telemetry (TelemetryProtocol | None): The telemetry service for
                tracking serialization events.
        """
        self._telemetry = telemetry

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> None:
        """Serializes a payload and writes it to standard output.

        Args:
            payload (Any): The object to serialize and emit.
            fmt (OutputFormat): The output format.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            None:
        """
        sys.stdout.write(self.dumps(payload, fmt=fmt, pretty=pretty))
        if not sys.stdout.isatty():
            sys.stdout.write("\n")
        sys.stdout.flush()

    @abc.abstractmethod
    def dumps(
        self,
        obj: Any,
        *,
        fmt: OutputFormat,
        pretty: bool = False,
    ) -> str:
        """Serializes an object to a string.

        Args:
            obj (Any): The object to serialize.
            fmt (OutputFormat): The desired output format.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            str: The serialized string.
        """
        ...

    @abc.abstractmethod
    def dumps_bytes(
        self,
        obj: Any,
        *,
        fmt: OutputFormat,
        pretty: bool = False,
    ) -> bytes:
        """Serializes an object to bytes.

        Args:
            obj (Any): The object to serialize.
            fmt (OutputFormat): The desired output format.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            bytes: The serialized bytes.
        """
        ...

    @abc.abstractmethod
    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat,
        pretty: bool = False,
    ) -> Any:
        """Deserializes data into a Python object.

        Args:
            data (str | bytes): The string or bytes to deserialize.
            fmt (OutputFormat): The format of the input data.
            pretty (bool): A hint that may affect parsing (often unused).

        Returns:
            Any: The deserialized object.
        """
        ...

    def _event(self, name: str, **data: Any) -> None:
        """Sends a telemetry event if the telemetry service is configured.

        Args:
            name (str): The name of the event.
            **data (Any): Additional data for the event payload.

        Returns:
            None:
        """
        if self._telemetry is not None:
            self._telemetry.event(name, data)

    @staticmethod
    def _axerr(fmt: OutputFormat, action: str, exc: Exception) -> BijuxError:
        """Creates a standardized `BijuxError` for serialization failures.

        Args:
            fmt (OutputFormat): The format that was being processed.
            action (str): The action being performed (e.g., "serialize").
            exc (Exception): The original exception that was caught.

        Returns:
            BijuxError: The wrapped error.
        """
        return BijuxError(f"Failed to {action} {fmt.value}: {exc}")


class OrjsonSerializer(_Base):
    """A serializer that uses `orjson` for JSON and `PyYAML` for YAML.

    This implementation prioritizes performance by using `orjson` for JSON
    operations if it is installed, gracefully falling back to the standard
    library's `json` module if it is not.
    """

    @staticmethod
    def _default(obj: Any) -> Any:
        """Provides a default function for JSON serialization.

        This handler allows custom types like `Redacted` to be serialized.

        Args:
            obj (Any): The object being serialized.

        Returns:
            Any: A serializable representation of the object.

        Raises:
            TypeError: If the object is not of a known custom type and is
                not otherwise JSON serializable.
        """
        if isinstance(obj, Redacted):
            return str(obj)
        raise TypeError(f"{obj!r} is not JSON serialisable")

    @staticmethod
    def _yaml_dump(obj: Any, pretty: bool) -> str:
        """Dumps an object to a YAML string.

        Args:
            obj (Any): The object to serialize.
            pretty (bool): If True, formats the output in block style.

        Returns:
            str: The serialized YAML string.

        Raises:
            BijuxError: If the `PyYAML` library is not installed.
        """
        if not _HAS_YAML:
            raise BijuxError("PyYAML is required for YAML operations")
        assert _YAML is not None  # noqa: S101  # nosec B101
        dumped = _YAML.safe_dump(
            obj,
            sort_keys=False,
            default_flow_style=not pretty,
            indent=2 if pretty else None,
        )
        return dumped or ""

    def _json_dump(self, obj: Any, pretty: bool) -> str | bytes:
        """Dumps an object to a JSON string or bytes, preferring `orjson`.

        Args:
            obj (Any): The object to serialize.
            pretty (bool): If True, indents the output.

        Returns:
            str | bytes: The serialized JSON data. Returns `bytes` if `orjson`
                is used, otherwise returns `str`.
        """
        if _HAS_ORJSON:
            assert _ORJSON is not None  # noqa: S101 # nosec B101
            opts = _ORJSON.OPT_INDENT_2 if pretty else 0
            raw = _ORJSON.dumps(obj, option=opts, default=self._default)
            return typing.cast(bytes, raw)  # pyright: ignore[reportUnnecessaryCast]
        return json.dumps(
            obj,
            indent=2 if pretty else None,
            ensure_ascii=False,
            default=self._default,
        )

    def dumps(
        self,
        obj: Any,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> str:
        """Serializes an object to a string.

        Args:
            obj (Any): The object to serialize.
            fmt (OutputFormat): The desired output format.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            str: The serialized string.

        Raises:
            BijuxError: If the format is unsupported or serialization fails.
        """
        try:
            if fmt is OutputFormat.JSON:
                raw = self._json_dump(obj, pretty)
                res = raw if isinstance(raw, str) else raw.decode()
            elif fmt is OutputFormat.YAML:
                res = self._yaml_dump(obj, pretty)
            else:
                raise BijuxError(f"Unsupported format: {fmt}")
            self._event("serialize_dumps", format=fmt.value, pretty=pretty)
            return res
        except Exception as exc:
            self._event("serialize_dumps_failed", format=fmt.value, error=str(exc))
            raise self._axerr(fmt, "serialize", exc) from exc

    def dumps_bytes(
        self,
        obj: Any,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> bytes:
        """Serializes an object to bytes.

        Args:
            obj (Any): The object to serialize.
            fmt (OutputFormat): The desired output format.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            bytes: The serialized bytes.

        Raises:
            BijuxError: If the format is unsupported or serialization fails.
        """
        try:
            if fmt is OutputFormat.JSON:
                raw = self._json_dump(obj, pretty)
                res = raw if isinstance(raw, bytes) else raw.encode()
            elif fmt is OutputFormat.YAML:
                res = self.dumps(obj, fmt=fmt, pretty=pretty).encode()
            else:
                raise BijuxError(f"Unsupported format: {fmt}")
            self._event("serialize_dumps_bytes", format=fmt.value, pretty=pretty)
            return res
        except Exception as exc:
            self._event(
                "serialize_dumps_bytes_failed",
                format=fmt.value,
                error=str(exc),
            )
            raise self._axerr(fmt, "serialize", exc) from exc

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> Any:
        """Deserializes data into a Python object.

        Args:
            data (str | bytes): The string or bytes to deserialize.
            fmt (OutputFormat): The format of the input data.
            pretty (bool): A hint that may affect parsing (often unused).

        Returns:
            Any: The deserialized object.

        Raises:
            BijuxError: If the format is unsupported or deserialization fails.
        """
        try:
            if fmt is OutputFormat.JSON:
                if _HAS_ORJSON:
                    assert _ORJSON is not None  # noqa: S101 # nosec B101
                    res = _ORJSON.loads(data)
                else:
                    res = json.loads(data)
            elif fmt is OutputFormat.YAML:
                if not _HAS_YAML:
                    raise BijuxError("PyYAML is required for YAML operations")
                assert _YAML is not None  # noqa: S101 # nosec B101
                txt = data if isinstance(data, str) else data.decode()
                res = _YAML.safe_load(txt) or {}
            else:
                raise BijuxError(f"Unsupported format: {fmt}")
            self._event("serialize_loads", format=fmt.value)
            return res
        except Exception as exc:
            self._event("serialize_loads_failed", format=fmt.value, error=str(exc))
            raise self._axerr(fmt, "deserialize", exc) from exc


class PyYAMLSerializer(_Base):
    """A serializer that exclusively uses the `PyYAML` library for YAML.

    Attributes:
        _patched (bool): A class-level flag to ensure that custom YAML
            representers are only registered once.
    """

    _patched = False

    @inject
    def __init__(self, telemetry: TelemetryProtocol | None) -> None:
        """Initializes the `PyYAMLSerializer`.

        This also registers a custom YAML representer for the `Redacted` type
        on first instantiation.

        Args:
            telemetry (TelemetryProtocol | None): The telemetry service.

        Raises:
            BijuxError: If the `PyYAML` library is not installed.
        """
        if not _HAS_YAML:
            raise BijuxError("PyYAML is not installed")
        super().__init__(telemetry)
        if not PyYAMLSerializer._patched:
            assert _YAML is not None  # noqa: S101 # nosec B101
            _YAML.add_representer(
                Redacted,
                lambda dumper, data: dumper.represent_scalar(
                    "tag:yaml.org,2002:str", str(data)
                ),
                Dumper=_YAML.SafeDumper,
            )
            PyYAMLSerializer._patched = True

    def dumps(
        self,
        obj: Any,
        *,
        fmt: OutputFormat = OutputFormat.YAML,
        pretty: bool = False,
    ) -> str:
        """Serializes an object to a YAML string.

        Args:
            obj (Any): The object to serialize.
            fmt (OutputFormat): The output format. Must be `OutputFormat.YAML`.
            pretty (bool): If True, formats the output in block style.

        Returns:
            str: The serialized YAML string.

        Raises:
            BijuxError: If the format is not `OutputFormat.YAML`.
        """
        if fmt is not OutputFormat.YAML:
            raise BijuxError("PyYAMLSerializer only supports YAML")
        return yaml_dump(obj, pretty)

    def dumps_bytes(
        self,
        obj: Any,
        *,
        fmt: OutputFormat = OutputFormat.YAML,
        pretty: bool = False,
    ) -> bytes:
        """Serializes an object to YAML bytes.

        Args:
            obj (Any): The object to serialize.
            fmt (OutputFormat): The output format. Must be `OutputFormat.YAML`.
            pretty (bool): If True, formats the output in block style.

        Returns:
            bytes: The serialized YAML bytes.
        """
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode()

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat = OutputFormat.YAML,
        pretty: bool = False,
    ) -> Any:
        """Deserializes YAML data into a Python object.

        Args:
            data (str | bytes): The string or bytes to deserialize.
            fmt (OutputFormat): The format of the input. Must be `OutputFormat.YAML`.
            pretty (bool): A hint that may affect parsing (unused).

        Returns:
            Any: The deserialized object.

        Raises:
            BijuxError: If the format is not `OutputFormat.YAML`.
        """
        if fmt is not OutputFormat.YAML:
            raise BijuxError("PyYAMLSerializer only supports YAML")
        txt = data if isinstance(data, str) else data.decode()
        assert _YAML is not None  # noqa: S101 # nosec B101
        return _YAML.safe_load(txt) or {}


if TYPE_CHECKING:
    from bijux_cli.contracts import SerializerProtocol


def serializer_for(
    fmt: OutputFormat | str,
    telemetry: TelemetryProtocol,
) -> SerializerProtocol[Any]:
    """A factory function that returns a serializer for the given format.

    Args:
        fmt (OutputFormat | str): The desired output format.
        telemetry (TelemetryProtocol): The telemetry service to inject into
            the serializer.

    Returns:
        SerializerProtocol[Any]: A configured serializer instance appropriate
            for the specified format.
    """
    format_enum = fmt if isinstance(fmt, OutputFormat) else OutputFormat(fmt.upper())

    if format_enum is OutputFormat.JSON:
        return OrjsonSerializer(telemetry)
    else:
        return PyYAMLSerializer(telemetry)


__all__ = [
    "Redacted",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "serializer_for",
]
