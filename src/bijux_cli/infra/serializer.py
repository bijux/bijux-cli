# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Serialization adapters for JSON and YAML formats."""

from __future__ import annotations

import importlib.util as _importlib_util
import json
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

    def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
        """Serialize an object to JSON or YAML."""
        if fmt is OutputFormat.JSON:
            try:
                if _ORJSON is not None:
                    option = _ORJSON.OPT_INDENT_2 if pretty else 0
                    return cast(
                        str,
                        _ORJSON.dumps(obj, option=option).decode("utf-8"),
                    )
                return json.dumps(obj, indent=2 if pretty else None)
            except Exception as exc:
                raise SerializationError(f"Failed to serialize json: {exc}") from exc
        if fmt is OutputFormat.YAML:
            return _yaml_dump(obj, pretty)
        raise SerializationError(f"Unsupported format: {fmt}")

    def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
        """Serialize an object to bytes."""
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat,
        pretty: bool,
    ) -> Any:
        """Deserialize JSON or YAML data."""
        if fmt is OutputFormat.JSON:
            try:
                return json.loads(data)
            except Exception as exc:
                raise SerializationError(f"Failed to deserialize json: {exc}") from exc
        if fmt is OutputFormat.YAML:
            if _YAML is None:
                raise SerializationError("PyYAML is required for YAML operations")
            return _YAML.safe_load(data)
        raise SerializationError(f"Unsupported format: {fmt}")


class PyYAMLSerializer:
    """Serializer restricted to YAML format."""

    def __init__(self, telemetry: Any | None) -> None:
        """Initialize with telemetry."""
        if _YAML is None:
            raise SerializationError("PyYAML is not installed")
        self._telemetry = telemetry

    def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
        """Serialize an object to YAML."""
        if fmt is not OutputFormat.YAML:
            raise SerializationError("PyYAMLSerializer only supports YAML")
        return _yaml_dump(obj, pretty)

    def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
        """Serialize an object to bytes."""
        return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat,
        pretty: bool,
    ) -> Any:
        """Deserialize YAML data."""
        if fmt is not OutputFormat.YAML:
            raise SerializationError("PyYAMLSerializer only supports YAML")
        return _YAML.safe_load(data) if _YAML is not None else None


def serializer_for(
    fmt: OutputFormat, telemetry: Any | None
) -> OrjsonSerializer | PyYAMLSerializer:
    """Return the best serializer for the requested format."""
    if fmt is OutputFormat.JSON:
        return OrjsonSerializer(telemetry)
    if fmt is OutputFormat.YAML:
        return PyYAMLSerializer(telemetry)
    raise SerializationError(f"Unsupported format: {fmt}")


__all__ = [
    "SerializationError",
    "OrjsonSerializer",
    "PyYAMLSerializer",
    "serializer_for",
]
