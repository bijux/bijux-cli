# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a persistent, cross-process safe command history service.

This module defines the `History` class, a concrete implementation of the
`HistoryProtocol`. It provides a tolerant and robust store for CLI
invocation events with several key design features:

    * **Persistence:** All history is saved to a single JSON array in a
        per-user file.
    * **Tolerance:** The service is resilient to empty, corrupt, or partially
        formed history files. If a file is unreadable, it is treated as empty
        and will be overwritten on the next successful write.
    * **Cross-Process Safety:** On POSIX systems, it uses `fcntl.flock` on a
        sidecar lock file to safely coordinate writes from multiple concurrent
        CLI processes. On other systems, it falls back to a thread lock.
    * **Atomic Writes:** All changes are written to a temporary file which is
        then atomically moved into place, preventing data corruption from
        interrupted writes.
    * **Memory Management:** The in-memory list of events is capped, and the
        on-disk file is trimmed to a smaller size to prevent unbounded growth.
    * **Simplicity:** The service intentionally avoids complex features like
        schema migrations. Unreadable state is discarded rather than repaired.
"""

from __future__ import annotations

from collections.abc import Iterator, MutableSequence, Sequence
from contextlib import contextmanager, suppress
import errno
import json
import os
from pathlib import Path
import sys
import tempfile
import threading
import time
from typing import Any, Final
import unicodedata

from injector import inject

from bijux_cli.contracts import HistoryProtocol
from bijux_cli.core.paths import HISTORY_FILE
from bijux_cli.infra.observability import Observability
from bijux_cli.infra.telemetry import LoggingTelemetry

_MAX_IN_MEMORY: Final[int] = 10_000
"""Maximum number of entries retained in memory (and considered for writes)."""
_TRIM_THRESHOLD: Final[int] = 1_000
"""When persisting, keep at most this many most-recent events in the file."""
_ENOSPC_ERRORS = {errno.ENOSPC, errno.EDQUOT}
"""OS error codes indicating the filesystem is full or quota exceeded."""
_FILE_LOCK = threading.Lock()
"""Fallback lock for non-POSIX platforms when `fcntl` is unavailable."""
fcntl: Any
try:
    import fcntl
except ImportError:  # pragma: no cover
    fcntl = None


def _now() -> float:
    """Returns the current UNIX time with sub-second precision.

    Returns:
        float: The current time in seconds since the epoch.
    """
    return time.time()


def _ascii_clean(text: str) -> str:
    """Strips all diacritics and non-printable characters from a string.

    Args:
        text (str): The input text to clean.

    Returns:
        str: An ASCII-only version of the text.
    """
    normalized = unicodedata.normalize("NFKD", text)
    without_marks = "".join(ch for ch in normalized if unicodedata.category(ch) != "Mn")
    return "".join(ch for ch in without_marks if 0x20 <= ord(ch) <= 0x7E)


def _lock_file_for(fp: Path) -> Path:
    """Returns the path for the sidecar lock file associated with `fp`.

    Args:
        fp (Path): The primary file path.

    Returns:
        Path: The corresponding lock file path (e.g., `file.lock`).
    """
    return fp.with_name(fp.name + ".lock")


@contextmanager
def _interprocess_lock(fp: Path) -> Iterator[None]:
    """Provides a cross-process exclusive lock for a file path.

    On POSIX systems, this uses `fcntl.flock` on a sidecar file to serialize
    access across different processes. On other platforms, it falls back to a
    `threading.Lock`, which only provides safety within a single process.

    Args:
        fp (Path): The path to the file that requires locked access.

    Yields:
        None: Yields control to the `with` block while the lock is held.
    """
    if fcntl is None:
        with _FILE_LOCK:
            yield
        return
    lock_fp = _lock_file_for(fp)
    lock_fp.parent.mkdir(parents=True, exist_ok=True)
    f = lock_fp.open("a+")
    try:
        fcntl.flock(f.fileno(), fcntl.LOCK_EX)
        yield
    finally:
        with suppress(Exception):
            fcntl.flock(f.fileno(), fcntl.LOCK_UN)
        f.close()


def _maybe_simulate_disk_full() -> None:
    """Raises an `ENOSPC` error if a test environment variable is set."""
    if os.getenv("BIJUXCLI_TEST_DISK_FULL") == "1":
        raise OSError(errno.ENOSPC, "No space left on device")


def _atomic_write_json(fp: Path, events: list[dict[str, Any]]) -> None:
    """Writes a list of events to a file atomically.

    The data is written to a temporary file in the same directory and then
    renamed to the final destination, which is an atomic operation on POSIX
    systems.

    Args:
        fp (Path): The destination file path.
        events (list[dict[str, Any]]): The list of history entries to write.

    Raises:
        PermissionError: If the directory or file is not writable.
        OSError: For other filesystem errors, such as a full disk.
    """
    _maybe_simulate_disk_full()
    fp.parent.mkdir(parents=True, exist_ok=True)
    to_write = events[-_TRIM_THRESHOLD:] if events else []
    payload = (
        "[]\n" if not to_write else json.dumps(to_write, ensure_ascii=False, indent=2)
    )
    with tempfile.NamedTemporaryFile(
        "w", delete=False, dir=fp.parent, prefix=f".{fp.name}.", encoding="utf-8"
    ) as temp_file:
        temp_file.write(payload)
        temp_file.flush()
        os.fsync(temp_file.fileno())
        temp_fp = Path(temp_file.name)
        os.replace(temp_fp, fp)


class History(HistoryProtocol):
    """Manages a persistent history of CLI command invocations.

    This service maintains an in-memory list of command events and synchronizes
    it with a persisted JSON file. It is designed to be tolerant of file
    corruption and safe for concurrent use by multiple CLI processes.

    Mutating operations (`add`, `clear`, `import_`) acquire a cross-process lock
    before modifying the file to prevent lost updates and race conditions. The
    sequence is always: lock, reload from disk, apply change in memory, write
    atomically, and release lock.

    Attributes:
        _tel (LoggingTelemetry): The telemetry service for emitting events.
        _obs (Observability): The logging service for operational errors.
        _explicit_path (Path | None): A specific path to the history file, if
            provided during initialization.
        _events (list): The in-memory cache of history event dictionaries.
        _load_error (str | None): A message describing the last error that
            occurred while trying to load the history file, if any.
    """

    @inject
    def __init__(
        self,
        telemetry: LoggingTelemetry,
        observability: Observability,
        history_path: Path | None = None,
    ) -> None:
        """Initializes the History service.

        Args:
            telemetry (LoggingTelemetry): The telemetry service.
            observability (Observability): The logging service.
            history_path (Path | None): An optional, explicit path to the
                history file. If None, a default path will be used.
        """
        self._tel = telemetry
        self._obs = observability
        self._explicit_path = Path(history_path) if history_path else None
        self._events: list[dict[str, Any]] = []
        self._load_error: str | None = None

    def _get_history_path(self) -> Path:
        """Returns the resolved, absolute path to the history file.

        The path is determined in the following order of precedence:
        1.  An explicit path provided to the constructor.
        2.  The `BIJUXCLI_HISTORY_FILE` environment variable.
        3.  A `.bijux_history` file in the same directory as the `BIJUXCLI_CONFIG` file.
        4.  The default `~/.bijux/.history` file.

        Returns:
            Path: The absolute path to the history file.
        """
        if self._explicit_path:
            return self._explicit_path
        env_file = os.environ.get("BIJUXCLI_HISTORY_FILE")
        if env_file:
            return Path(env_file).expanduser()
        cfg = os.environ.get("BIJUXCLI_CONFIG")
        if cfg:
            cfg_path = Path(cfg).expanduser()
            return cfg_path.parent / ".bijux_history"
        return HISTORY_FILE

    def _reload(self) -> None:
        """Refreshes the in-memory state from the history file on disk.

        This method is tolerant of errors. If the file is missing, empty, or
        corrupt, the in-memory list is cleared and an error state is noted,
        but an exception is not raised.
        """
        self._load_error = None
        fp = self._get_history_path()
        try:
            if not fp.exists():
                self._events = []
                return
            raw = fp.read_text(encoding="utf-8", errors="ignore").strip()
            if not raw:
                self._events = []
                return
            data = json.loads(raw)
            if not isinstance(data, list):
                self._events = []
                self._load_error = (
                    f"Unexpected history file format (not JSON array): {fp}"
                )
                return
            evs: list[dict[str, Any]] = []
            for item in data:
                if not isinstance(item, dict):
                    continue
                e = dict(item)
                e["command"] = _ascii_clean(str(e.get("command", "")))
                evs.append(e)
            if len(evs) > _MAX_IN_MEMORY:
                evs = evs[-_MAX_IN_MEMORY:]
            self._events = evs
        except Exception as exc:
            self._load_error = f"History file corrupted or unreadable: {exc}"
            self._obs.log("error", self._load_error, extra={"path": str(fp)})
            self._events = []

    def _dump(self) -> None:
        """Persists the current in-memory events to disk atomically."""
        fp = self._get_history_path()
        with _interprocess_lock(fp):
            self._load_error = None
            try:
                _atomic_write_json(fp, self._events)
            except PermissionError as exc:
                self._handle_dump_error("write-permission", exc, fp)
                raise
            except OSError as exc:
                if exc.errno in _ENOSPC_ERRORS:
                    self._handle_dump_error("persist", exc, fp)
                    raise
                raise

    def _handle_dump_error(self, kind: str, exc: OSError, fp: Path) -> None:
        """Logs and prints an error encountered during a file write operation.

        Args:
            kind (str): A short code classifying the error (e.g., "persist").
            exc (OSError): The originating exception.
            fp (Path): The path of the file that was being written to.
        """
        msg = f"History {kind} error: {exc}"
        self._obs.log("error", msg, extra={"path": str(fp)})
        self._load_error = msg
        print(msg, file=sys.stderr)

    def add(
        self,
        command: str,
        *,
        params: Sequence[str] | None = None,
        success: bool | None = True,
        return_code: int | None = 0,
        duration_ms: float | None = None,
    ) -> None:
        """Appends a new command invocation to the history.

        This operation is cross-process safe. It acquires a lock, reloads the
        latest history from disk, appends the new entry, and writes the
        updated history back atomically. Errors are logged but suppressed to
        allow the originating command to complete its execution.

        Args:
            command (str): The command name (ASCII characters are enforced).
            params (Sequence[str] | None): A list of parameters and flags.
            success (bool | None): Whether the command succeeded.
            return_code (int | None): The exit code of the command.
            duration_ms (float | None): The command's duration in milliseconds.
        """
        fp = self._get_history_path()
        entry = {
            "command": _ascii_clean(command),
            "params": list(params or []),
            "timestamp": _now(),
            "success": bool(success),
            "return_code": return_code if return_code is not None else 0,
            "duration_ms": float(duration_ms) if duration_ms is not None else None,
        }
        with _interprocess_lock(fp):
            self._reload()
            if self._load_error:
                msg = f"[error] Could not load command history: {self._load_error}"
                self._obs.log("error", msg, extra={"path": str(fp)})
                print(msg, file=sys.stderr)
                self._events = []
            self._events.append(entry)
            try:
                _atomic_write_json(fp, self._events)
                self._load_error = None
            except PermissionError as exc:
                msg = f"[error] Could not record command history: {exc}"
                self._obs.log("error", msg, extra={"path": str(fp)})
                print(msg, file=sys.stderr)
                self._load_error = msg
                return
            except OSError as exc:
                if exc.errno in _ENOSPC_ERRORS:
                    msg = f"[error] Could not record command history: {exc}"
                    self._obs.log("error", msg, extra={"path": str(fp)})
                    print(msg, file=sys.stderr)
                    self._load_error = msg
                    return
                msg = f"[error] Could not record command history: {exc}"
                self._obs.log("error", msg, extra={"path": str(fp)})
                print(msg, file=sys.stderr)
                self._load_error = msg
                return
        with suppress(Exception):
            self._tel.event("history_event_added", {"command": entry["command"]})

    def list(
        self,
        *,
        limit: int | None = 20,
        group_by: str | None = None,
        filter_cmd: str | None = None,
        sort: str | None = None,
    ) -> list[dict[str, Any]]:
        """Returns a view of the command history, with optional transformations.

        This is a read-only operation and does not acquire a cross-process lock,
        meaning it may not reflect writes from concurrent processes.

        Args:
            limit (int | None): The maximum number of entries to return. A value
                of 0 returns an empty list.
            group_by (str | None): If provided, returns a grouped summary.
            filter_cmd (str | None): If provided, returns only entries whose
                command contains this case-sensitive substring.
            sort (str | None): If 'timestamp', sorts entries by timestamp.

        Returns:
            list[dict[str, Any]]: A list of history entries or grouped summaries.

        Raises:
            RuntimeError: If the history file is corrupt.
        """
        self._reload()
        fp = self._get_history_path()
        try:
            writable = os.access(fp.parent, os.W_OK)
        except Exception:
            writable = True
        if not writable:
            msg = f"Permission denied for history directory: {fp.parent}"
            self._obs.log("error", msg, extra={"path": str(fp)})
            print(msg, file=sys.stderr)
        if self._load_error:
            raise RuntimeError(self._load_error)
        if limit == 0:
            return []
        entries: list[dict[str, Any]] = list(self._events)
        if filter_cmd:
            needle = str(filter_cmd)
            entries = [e for e in entries if needle in (e.get("command") or "")]
        if sort == "timestamp":
            entries.sort(key=lambda e: e.get("timestamp", 0))
        if group_by:
            grouped: dict[Any, MutableSequence[dict[str, Any]]] = {}
            for e in entries:
                grouped.setdefault(e.get(group_by, "unknown"), []).append(e)
            summary = [
                {
                    "group": k,
                    "count": len(v),
                    "last_run": max((x.get("timestamp", 0) for x in v), default=0),
                }
                for k, v in grouped.items()
            ]
            return summary[:limit] if (limit and limit > 0) else summary
        if limit and limit > 0:
            entries = entries[-limit:]
        return entries

    def clear(self) -> None:
        """Erases all persisted history.

        This operation is cross-process safe and atomic.

        Raises:
            PermissionError: If the history file or directory is not writable.
            OSError: For other filesystem-related failures.
        """
        fp = self._get_history_path()
        try:
            with _interprocess_lock(fp):
                self._events = []
                _atomic_write_json(fp, self._events)
                self._load_error = None
                self._tel.event("history_cleared", {})
        except Exception as exc:
            msg = f"History clear failed: {exc}"
            self._obs.log("error", msg, extra={"path": str(fp)})
            self._load_error = msg
            raise
        finally:
            self._reload()

    def flush(self) -> None:
        """Persists all in-memory history data to disk."""
        self._dump()

    def export(self, path: Path) -> None:
        """Exports the current history to a file as a JSON array.

        This operation is a read-only snapshot and does not lock the source file.

        Args:
            path (Path): The destination file path.

        Raises:
            RuntimeError: On I/O failures.
        """
        self._reload()
        try:
            path = path.expanduser()
            path.parent.mkdir(parents=True, exist_ok=True)
            text = json.dumps(self._events, ensure_ascii=False, indent=2) + "\n"
            path.write_text(text, encoding="utf-8")
        except Exception as exc:
            raise RuntimeError(f"Failed exporting history: {exc}") from exc

    def import_(self, path: Path) -> None:
        """Imports history entries from a file, merging with current history.

        This operation is cross-process safe and atomic.

        Args:
            path (Path): The source file path containing a JSON array of entries.

        Raises:
            RuntimeError: On I/O or parsing failures.
        """
        fp = self._get_history_path()
        try:
            with _interprocess_lock(fp):
                self._reload()
                if self._load_error:
                    raise RuntimeError(self._load_error)
                path = path.expanduser()
                if not path.exists():
                    raise RuntimeError(f"Import file not found: {path}")
                raw = path.read_text(encoding="utf-8")
                data = json.loads(raw)
                if not isinstance(data, list):
                    raise RuntimeError(
                        f"Invalid import format (not JSON array): {path}"
                    )
                imported: list[dict[str, Any]] = []
                for item in data:
                    if not isinstance(item, dict):
                        continue
                    e = dict(item)
                    e["command"] = _ascii_clean(str(e.get("command", "")))
                    if "timestamp" not in e:
                        e["timestamp"] = _now()
                    imported.append(e)
                self._events.extend(imported)
                if len(self._events) > _MAX_IN_MEMORY:
                    self._events = self._events[-_MAX_IN_MEMORY:]
                _atomic_write_json(fp, self._events)
                self._load_error = None
                with suppress(Exception):
                    self._tel.event("history_imported", {"count": len(imported)})

        except Exception as exc:
            msg = f"History import failed: {exc}"
            self._obs.log(
                "error", msg, extra={"import_path": str(path), "history_path": str(fp)}
            )
            raise RuntimeError(msg) from exc


__all__ = ["History"]
