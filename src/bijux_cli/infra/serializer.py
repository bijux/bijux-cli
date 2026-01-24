# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Serialization adapters for JSON and YAML formats."""

from __future__ import annotations

import importlib.util as _importlib_util
import json
import sys
from types import ModuleType
from typing import Any, Final, cast

from bijux_cli.core.enums import OutputFormat

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
    import yaml as _yaml_mod
except ImportError:
    _yaml_mod = None
_YAML: Final[ModuleType | None] = _yaml_mod


class SerializationError(RuntimeError):
    """Raised when serialization or deserialization fails."""


def _redact_default(_: Any) -> str:
    """Fallback serializer for non-JSON values."""
    return "***"


def _format_name(fmt: Any) -> str:
    """Normalize format values to lowercase strings."""
    if hasattr(fmt, "value"):
        return str(fmt.value).lower()
    if isinstance(fmt, str):
        return fmt.lower()
    return str(fmt).lower()


def _yaml_dump(obj: Any, pretty: bool) -> str:
    """Serialize an object to YAML."""
    if _YAML is None:
        raise SerializationError("PyYAML is required for YAML operations")
    dumped = _YAML.safe_dump(
        obj,
        sort_keys=False,
        default_flow_style=not pretty,
        indent=2 if pretty else None,
    )
    return dumped or ""


class OrjsonSerializer:
    """Serializer that handles JSON (and YAML via PyYAML)."""

    def __init__(self, telemetry: Any | None) -> None:
        """Initialize with telemetry."""
        self._telemetry = telemetry

    def dumps(
        self, obj: Any, *, fmt: OutputFormat = OutputFormat.JSON, pretty: bool = False
    ) -> str:
        """Serialize an object to JSON or YAML."""
        name = _format_name(fmt)
        if name == "json":
            try:
                if _ORJSON is not None:
                    option = _ORJSON.OPT_INDENT_2 if pretty else 0
                    return cast(
                        str,
                        _ORJSON.dumps(
                            obj, default=_redact_default, option=option
                        ).decode("utf-8"),
                    )
                return json.dumps(obj, indent=2 if pretty else None)
            except Exception as exc:
                raise SerializationError(f"Failed to serialize json: {exc}") from exc
        if name == "yaml":
            return _yaml_dump(obj, pretty)
        raise SerializationError(f"Unsupported format: {fmt}")

    def dumps_bytes(
        self, obj: Any, *, fmt: OutputFormat = OutputFormat.JSON, pretty: bool = False
    ) -> bytes:
        """Serialize an object to bytes."""
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> Any:
        """Deserialize JSON or YAML data."""
        name = _format_name(fmt)
        if name == "json":
            try:
                return json.loads(data)
            except Exception as exc:
                raise SerializationError(f"Failed to deserialize json: {exc}") from exc
        if name == "yaml":
            if _YAML is None:
                raise SerializationError("PyYAML is required for YAML operations")
            return _YAML.safe_load(data)
        raise SerializationError(f"Unsupported format: {fmt}")

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> None:
        """Serialize and print to stdout."""
        text = self.dumps(payload, fmt=fmt, pretty=pretty)
        print(text.rstrip("\n"), file=sys.stdout, flush=True)
        if self._telemetry is not None:
            self._telemetry.event("serializer_emit", {"format": _format_name(fmt)})


class PyYAMLSerializer:
    """Serializer restricted to YAML format."""

    def __init__(self, telemetry: Any | None) -> None:
        """Initialize with telemetry."""
        if _YAML is None:
            raise SerializationError("PyYAML is not installed")
        self._telemetry = telemetry

    def dumps(
        self, obj: Any, *, fmt: OutputFormat = OutputFormat.YAML, pretty: bool = False
    ) -> str:
        """Serialize an object to YAML."""
        if _format_name(fmt) != "yaml":
            raise SerializationError("PyYAMLSerializer only supports YAML")
        return _yaml_dump(obj, pretty)

    def dumps_bytes(
        self, obj: Any, *, fmt: OutputFormat = OutputFormat.YAML, pretty: bool = False
    ) -> bytes:
        """Serialize an object to bytes."""
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat = OutputFormat.YAML,
        pretty: bool = False,
    ) -> Any:
        """Deserialize YAML data."""
        if _format_name(fmt) != "yaml":
            raise SerializationError("PyYAMLSerializer only supports YAML")
        return _YAML.safe_load(data) if _YAML is not None else None

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat = OutputFormat.YAML,
        pretty: bool = False,
    ) -> None:
        """Serialize and print to stdout."""
        text = self.dumps(payload, fmt=fmt, pretty=pretty)
        print(text.rstrip("\n"), file=sys.stdout, flush=True)
        if self._telemetry is not None:
            self._telemetry.event("serializer_emit", {"format": _format_name(fmt)})


def serializer_for(
    fmt: OutputFormat, telemetry: Any | None
) -> OrjsonSerializer | PyYAMLSerializer:
    """Return the best serializer for the requested format."""
    name = _format_name(fmt)
    if name == "json":
        return OrjsonSerializer(telemetry)
    if name == "yaml":
        return PyYAMLSerializer(telemetry)
    raise SerializationError(f"Unsupported format: {fmt}")


__all__ = [
    "SerializationError",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "serializer_for",
]
