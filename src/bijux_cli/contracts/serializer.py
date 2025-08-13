# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the object serialization service.

This module specifies the `SerializerProtocol`, a formal interface that any
service responsible for serializing objects to strings or bytes (e.g., in
JSON or YAML format) and deserializing them back must implement.
"""

from __future__ import annotations

from typing import Protocol, TypeVar, runtime_checkable

from bijux_cli.core.enums import OutputFormat

T = TypeVar("T")


@runtime_checkable
class SerializerProtocol(Protocol[T]):
    """Defines the contract for stateless, thread-safe object serialization.

    This interface specifies methods for serializing and deserializing objects
    to and from strings or bytes in various formats, such as JSON or YAML.
    """

    def dumps(
        self,
        obj: T,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> str:
        """Serializes an object to a string.

        Args:
            obj (T): The object to serialize.
            fmt (OutputFormat): The output format. Defaults to `OutputFormat.JSON`.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            str: The serialized object as a string.
        """
        ...

    def dumps_bytes(
        self,
        obj: T,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> bytes:
        """Serializes an object to bytes.

        Args:
            obj (T): The object to serialize.
            fmt (OutputFormat): The output format. Defaults to `OutputFormat.JSON`.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            bytes: The serialized object as bytes.
        """
        ...

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: OutputFormat = OutputFormat.JSON,
        pretty: bool = False,
    ) -> T:
        """Deserializes data from a string or bytes into an object.

        Args:
            data (str | bytes): The string or bytes to deserialize.
            fmt (OutputFormat): The format of the input data. Defaults to
                `OutputFormat.JSON`.
            pretty (bool): A hint that may affect parsing, though often unused
                during deserialization.

        Returns:
            T: The deserialized object.
        """
        ...

    def emit(
        self, payload: T, *, fmt: OutputFormat = OutputFormat.JSON, pretty: bool = False
    ) -> None:
        """Serializes and emits a payload to standard output.

        Args:
            payload (T): The object to serialize and emit.
            fmt (OutputFormat): The output format.
            pretty (bool): If True, formats the output for human readability.

        Returns:
            None:
        """
        ...
