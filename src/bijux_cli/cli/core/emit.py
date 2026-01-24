# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Emit structured payloads without exit behavior."""

from __future__ import annotations

import sys
from typing import Any

from bijux_cli.core.enums import OutputFormat
from bijux_cli.infra.contracts import Serializer


def resolve_serializer() -> Serializer:
    """Resolve the serializer adapter or fallback."""
    import json

    from bijux_cli.core.di import DIContainer

    class _FallbackSerializer:
        """Minimal JSON serializer when DI is unavailable."""

        def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
            _ = fmt
            return json.dumps(obj, indent=2 if pretty else None, default=str)

        def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
            return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

        def loads(self, data: str | bytes, *, fmt: OutputFormat, pretty: bool) -> Any:
            _ = (fmt, pretty)
            if isinstance(data, bytes):
                data = data.decode("utf-8")
            return json.loads(data)

        def emit(self, payload: Any, *, fmt: OutputFormat, pretty: bool) -> None:
            print(self.dumps(payload, fmt=fmt, pretty=pretty).rstrip("\n"))

    try:
        serializer = DIContainer.current().resolve(Serializer)
        if not hasattr(serializer, "dumps"):
            raise RuntimeError("Serializer does not implement dumps()")
        return serializer
    except Exception:
        return _FallbackSerializer()


def emit_payload(
    payload: object, *, fmt: OutputFormat, pretty: bool, stream: str
) -> None:
    """Emit a payload to the requested stream."""
    out = sys.stdout if stream == "stdout" else sys.stderr
    try:
        output = (
            resolve_serializer().dumps(payload, fmt=fmt, pretty=pretty).rstrip("\n")
        )
        print(output, file=out, flush=True)
    except Exception:
        print('{"error": "Unserializable error"}', file=sys.stderr, flush=True)


__all__ = ["emit_payload", "resolve_serializer"]
