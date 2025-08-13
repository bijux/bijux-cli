# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the API specification writing service.

This module specifies the `DocsProtocol`, a formal interface that any
service responsible for generating and writing API specification documents
(e.g., OpenAPI, JSON Schema) must implement.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Protocol, TypeVar, runtime_checkable

from bijux_cli.core.enums import OutputFormat

T = TypeVar("T")


@runtime_checkable
class DocsProtocol(Protocol):
    """Defines the contract for writing API specifications.

    This interface specifies methods for serializing and writing specification
    documents, such as OpenAPI or JSON Schema, in various formats.
    """

    def write(
        self,
        spec: dict[str, Any],
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        name: str = "spec",
    ) -> str:
        """Writes a specification to a file.

        Args:
            spec (dict[str, Any]): The specification dictionary to write.
            fmt (OutputFormat): The output format. Defaults to `OutputFormat.JSON`.
            name (str): The base name for the output file. Defaults to 'spec'.

        Returns:
            str: The path to the written file as a string.
        """
        ...

    def write_sync(
        self, spec: dict[Any, Any], fmt: OutputFormat, name: str | Path
    ) -> Path:
        """Writes the specification to a file synchronously.

        Args:
            spec (dict[Any, Any]): The specification dictionary to write.
            fmt (OutputFormat): The output format (e.g., JSON, YAML).
            name (str | Path): The path or name for the output file.

        Returns:
            Path: The `Path` object pointing to the written file.
        """
        ...

    def close(self) -> None:
        """Closes the writer and releases any associated resources."""
        ...
