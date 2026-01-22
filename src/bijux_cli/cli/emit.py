# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Emit structured payloads and exit."""

from __future__ import annotations

from collections.abc import Mapping
import json
import logging
import sys
import time
from typing import Any, NoReturn

from bijux_cli.cli.validation import ascii_safe
from bijux_cli.core.contracts import Serializer
from bijux_cli.core.enums import OutputFormat


def resolve_serializer() -> Serializer:
    """Resolve the serializer adapter or fallback."""
    try:
        from bijux_cli.app.di import DIContainer

        serializer = DIContainer.current().resolve(Serializer)
        if hasattr(serializer, "dumps"):
            return serializer
    except Exception as exc:
        logging.getLogger(__name__).debug("Failed to resolve serializer", exc_info=exc)

    class _FallbackSerializer:
        def dumps(self, obj: Any, *, fmt: Any = "json", pretty: bool = False) -> str:
            if str(fmt).lower() == "yaml":
                try:
                    import yaml

                    return yaml.safe_dump(obj, sort_keys=False)
                except Exception:
                    return json.dumps(obj, indent=2 if pretty else None)
            return json.dumps(obj, indent=2 if pretty else None)

        def dumps_bytes(
            self, obj: Any, *, fmt: Any = "json", pretty: bool = False
        ) -> bytes:
            return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

        def loads(
            self, data: str | bytes, *, fmt: Any = "json", pretty: bool = False
        ) -> Any:
            _ = pretty
            if str(fmt).lower() == "yaml":
                try:
                    import yaml

                    return yaml.safe_load(data)
                except Exception:
                    return data
            return json.loads(data)

        def emit(
            self, payload: Any, *, fmt: Any = "json", pretty: bool = False
        ) -> None:
            sys.stdout.write(self.dumps(payload, fmt=fmt, pretty=pretty))
            sys.stdout.write("\n")

    return _FallbackSerializer()


def emit_and_exit(
    payload: Mapping[str, Any],
    fmt: OutputFormat,
    effective_pretty: bool,
    verbose: bool,
    debug: bool,
    quiet: bool,
    command: str,
    *,
    exit_code: int = 0,
) -> NoReturn:
    """Serialize payload, record history, and exit."""
    if (not quiet) and (not command.startswith("history")):
        try:
            from bijux_cli.app.di import DIContainer
            from bijux_cli.services.history.contracts import HistoryProtocol

            hist = DIContainer.current().resolve(HistoryProtocol)
            hist.add(
                command=command,
                params=[],
                success=(exit_code == 0),
                return_code=exit_code,
                duration_ms=0.0,
            )
        except PermissionError as exc:
            print(f"Permission denied writing history: {exc}", file=sys.stderr)
        except OSError as exc:
            import errno as _errno

            if exc.errno in (_errno.EACCES, _errno.EPERM):
                print(f"Permission denied writing history: {exc}", file=sys.stderr)
            elif exc.errno in (_errno.ENOSPC, _errno.EDQUOT):
                print(
                    f"No space left on device while writing history: {exc}",
                    file=sys.stderr,
                )
            else:
                print(f"Error writing history: {exc}", file=sys.stderr)
        except Exception as exc:
            print(f"Error writing history: {exc}", file=sys.stderr)

    if quiet:
        sys.exit(exit_code)

    if debug:
        print("Diagnostics: emitted payload", file=sys.stderr)

    output = resolve_serializer().dumps(payload, fmt=fmt, pretty=effective_pretty)
    print(output.rstrip("\n"))
    sys.exit(exit_code)


def emit_error_and_exit(
    message: str,
    code: int,
    failure: str,
    command: str | None = None,
    fmt: str | None = None,
    quiet: bool = False,
    include_runtime: bool = False,
    debug: bool = False,
    extra: dict[str, Any] | None = None,
) -> NoReturn:
    """Emit a structured error payload to stderr and exit."""
    if quiet:
        sys.exit(code)

    if debug:
        import traceback

        traceback.print_exc(file=sys.stderr)

    error_payload = {"error": message, "code": code}
    if failure:
        error_payload["failure"] = failure
    if command:
        error_payload["command"] = command
    if fmt:
        error_payload["fmt"] = fmt
    if extra:
        error_payload.update(extra)
    if include_runtime:
        error_payload["python"] = ascii_safe(sys.version.split()[0], "python_version")
        error_payload["platform"] = ascii_safe(sys.platform, "platform")
        error_payload["timestamp"] = str(time.time())

    serializer = resolve_serializer()
    try:
        output = serializer.dumps(
            error_payload,
            fmt=str(error_payload.get("format", "json")),
            pretty=False,
        ).rstrip("\n")
        print(output, file=sys.stderr, flush=True)
    except Exception:
        print('{"error": "Unserializable error"}', file=sys.stderr, flush=True)
    sys.exit(code)


__all__ = ["emit_and_exit", "emit_error_and_exit", "resolve_serializer"]
