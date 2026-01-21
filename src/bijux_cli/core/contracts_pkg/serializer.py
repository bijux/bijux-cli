# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the object serialization service.

This module specifies the `SerializerProtocol`, a formal interface that any
service responsible for serializing objects to strings or bytes (e.g., in
JSON or YAML format) and deserializing them back must implement.
"""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class SerializerProtocol(Protocol):
    """Defines the contract for stateless, thread-safe object serialization.

    This interface specifies methods for serializing and deserializing objects
    to and from strings or bytes in various formats, such as JSON or YAML.
    """

    def dumps(
        self,
        obj: Any,
        *,
        fmt: str,
        pretty: bool,
    ) -> str:
        """Serialize an object to a string."""
        ...

    def dumps_bytes(
        self,
        obj: Any,
        *,
        fmt: str,
        pretty: bool,
    ) -> bytes:
        """Serialize an object to bytes."""
        ...

    def loads(
        self,
        data: str | bytes,
        *,
        fmt: str,
        pretty: bool,
    ) -> Any:
        """Deserialize data to an object."""
        ...

    def emit(self, payload: Any, *, fmt: str, pretty: bool) -> None:
        """Serialize and emit a payload."""
        ...
