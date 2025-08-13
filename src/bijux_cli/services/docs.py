# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the concrete implementation of the API specification writing service.

This module defines the `Docs` class, which implements the `DocsProtocol`.
It is responsible for serializing API specification data into formats like
JSON or YAML and writing the resulting documents to the filesystem. It
integrates with observability and telemetry services to log its activities.
"""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any
from weakref import WeakKeyDictionary

from injector import inject

from bijux_cli.contracts import DocsProtocol, ObservabilityProtocol, TelemetryProtocol
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import ServiceError
from bijux_cli.infra.serializer import serializer_for


class Docs(DocsProtocol):
    """A service for writing API specification documents to disk.

    This class implements the `DocsProtocol` to handle the serialization and
    writing of specifications (e.g., OpenAPI, JSON Schema) to files. It
    maintains a cache of serializer instances for performance.

    Attributes:
        _serializers (WeakKeyDictionary): A cache of serializer instances, keyed
            by the telemetry service instance.
        _observability (ObservabilityProtocol): The logging service.
        _telemetry (TelemetryProtocol): The telemetry service for event tracking.
        _root (Path): The root directory where documents will be written.
    """

    _serializers: WeakKeyDictionary[TelemetryProtocol, dict[OutputFormat, Any]] = (
        WeakKeyDictionary()
    )

    @inject
    def __init__(
        self,
        observability: ObservabilityProtocol,
        telemetry: TelemetryProtocol,
        root: str | Path | None = None,
    ) -> None:
        """Initializes the `Docs` service.

        Args:
            observability (ObservabilityProtocol): The service for logging.
            telemetry (TelemetryProtocol): The service for event tracking.
            root (str | Path | None): The root directory for writing documents.
                It defaults to the `BIJUXCLI_DOCS_DIR` environment variable,
                or "docs" if not set.
        """
        self._observability = observability
        self._telemetry = telemetry
        env_root = os.getenv("BIJUXCLI_DOCS_DIR")
        root_dir = env_root if env_root else (root or "docs")
        self._root = Path(root_dir)
        self._root.mkdir(exist_ok=True, parents=True)
        if telemetry not in self._serializers:
            self._serializers[telemetry] = {}

    def render(self, spec: dict[str, Any], *, fmt: OutputFormat) -> str:
        """Renders a specification dictionary to a string in the given format.

        Args:
            spec (dict[str, Any]): The specification dictionary to serialize.
            fmt (OutputFormat): The desired output format (e.g., JSON, YAML).

        Returns:
            str: The serialized specification as a string.

        Raises:
            TypeError: If the underlying serializer returns a non-string result.
        """
        if self._telemetry not in self._serializers:
            self._serializers[self._telemetry] = {}
        if fmt not in self._serializers[self._telemetry]:
            self._serializers[self._telemetry][fmt] = serializer_for(
                fmt, self._telemetry
            )
        result = self._serializers[self._telemetry][fmt].dumps(
            spec, fmt=fmt, pretty=False
        )
        if not isinstance(result, str):
            raise TypeError(
                f"Expected str from serializer.dumps, got {type(result).__name__}"
            )
        return result

    def write(
        self,
        spec: dict[str, Any],
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        name: str = "spec",
    ) -> str:
        """Writes a specification to a file and returns the path as a string.

        This is a convenience wrapper around `write_sync`.

        Args:
            spec (dict[str, Any]): The specification dictionary to write.
            fmt (OutputFormat): The output format. Defaults to `OutputFormat.JSON`.
            name (str): The base name for the output file. Defaults to 'spec'.

        Returns:
            str: The absolute path to the written file.
        """
        path = self.write_sync(spec, fmt, name)
        return str(path)

    def write_sync(
        self, spec: dict[str, Any], fmt: OutputFormat, name: str | Path
    ) -> Path:
        """Writes the specification to a file synchronously.

        This method handles path resolution, serializes the `spec` dictionary,
        and writes the content to the final destination file.

        Args:
            spec (dict[str, Any]): The specification dictionary to write.
            fmt (OutputFormat): The desired output format.
            name (str | Path): The path or base name for the output file.

        Returns:
            Path: The `Path` object pointing to the written file.

        Raises:
            ServiceError: If writing to the file fails due to an `OSError`.
        """
        final_path = None
        try:
            final_path = Path(name).expanduser().resolve()
            if final_path.is_dir():
                final_path = final_path / f"spec.{fmt.value}"
            final_path.parent.mkdir(parents=True, exist_ok=True)
            content = self.render(spec, fmt=fmt)
            final_path.write_text(content, encoding="utf-8")
            self._observability.log("info", f"Wrote docs to {final_path}")
            self._telemetry.event(
                "docs_written", {"path": str(final_path), "format": fmt.value}
            )
            return final_path
        except OSError as exc:
            self._telemetry.event(
                "docs_write_failed",
                {
                    "path": (
                        str(final_path) if final_path is not None else "<unresolved>"
                    ),
                    "error": str(exc),
                },
            )
            raise ServiceError(f"Unable to write spec: {exc}", http_status=403) from exc

    def close(self) -> None:
        """Closes the service. This is a no-op for this implementation."""
        return


__all__ = ["Docs"]
