# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services history module."""

# pyright: reportPrivateUsage=false

from __future__ import annotations

import errno
import json
import os
from pathlib import Path
import threading
import time
from typing import Any
from unittest.mock import MagicMock, Mock, call, patch

import pytest

from bijux_cli.infra.observability import Observability
from bijux_cli.infra.telemetry import LoggingTelemetry
from bijux_cli.services.history import (
    _MAX_IN_MEMORY,
    _TRIM_THRESHOLD,
    History,
    _ascii_clean,
    _atomic_write_json,
    _interprocess_lock,
    _lock_file_for,
    _maybe_simulate_disk_full,
    _now,
)


@pytest.fixture
def mock_telemetry() -> LoggingTelemetry:
    """Provide a mock LoggingTelemetry instance."""
    return Mock(spec=LoggingTelemetry)


@pytest.fixture
def mock_observability() -> Observability:
    """Provide a mock Observability instance."""
    return Mock(spec=Observability)


@pytest.fixture
def temp_history_file(tmp_path: Path) -> Path:
    """Provide a temporary file path for the history store."""
    return tmp_path / "history.json"


@pytest.fixture
def history(
    mock_telemetry: LoggingTelemetry,
    mock_observability: Observability,
    temp_history_file: Path,
) -> History:
    """Provide a History instance with a temporary file path."""
    return History(
        telemetry=mock_telemetry,
        observability=mock_observability,
        history_path=temp_history_file,
    )


@pytest.fixture
def history_no_path(
    mock_telemetry: LoggingTelemetry, mock_observability: Observability
) -> History:
    """Provide a History instance without an explicit file path."""
    return History(telemetry=mock_telemetry, observability=mock_observability)


def test_now() -> None:
    """Test that _now returns a float close to the current time."""
    current = time.time()
    assert abs(_now() - current) < 1.0


@pytest.mark.parametrize(
    ("text", "expected"),
    [
        ("ascii", "ascii"),
        ("unicodé", "unicode"),
        ("", ""),
        ("\x00null", "null"),
    ],
)
def test_ascii_clean(text: str, expected: str) -> None:
    """Test that _ascii_clean removes non-ASCII and null characters."""
    assert _ascii_clean(text) == expected


def test_ascii_clean_invalid() -> None:
    """Test _ascii_clean with various Unicode and emoji characters."""
    assert _ascii_clean("abc") == "abc"
    assert _ascii_clean("Ångström") == "Angstrom"
    assert _ascii_clean("hello😀") == "hello"
    assert _ascii_clean("⚡🐍") == ""


def test_lock_file_for(temp_history_file: Path) -> None:
    """Test that _lock_file_for correctly appends the .lock suffix."""
    assert _lock_file_for(temp_history_file) == temp_history_file.with_name(
        temp_history_file.name + ".lock"
    )


@patch("bijux_cli.services.history.fcntl", None)
def test_interprocess_lock_non_posix(temp_history_file: Path) -> None:
    """Test that the interprocess lock falls back to a threading.Lock on non-POSiX systems."""
    with _interprocess_lock(temp_history_file):
        pass


@patch("bijux_cli.services.history.fcntl")
def test_interprocess_lock_posix(
    mock_fcntl: MagicMock, temp_history_file: Path
) -> None:
    """Test that the interprocess lock uses fcntl.flock on POSIX systems."""
    mock_file = MagicMock()
    mock_file.fileno.return_value = 99
    with (
        patch("pathlib.Path.open", return_value=mock_file),
        _interprocess_lock(temp_history_file),
    ):
        pass
    flock_calls = [
        call(mock_file.fileno(), mock_fcntl.LOCK_EX),
        call(mock_file.fileno(), mock_fcntl.LOCK_UN),
    ]
    actual_calls = mock_fcntl.flock.call_args_list
    assert actual_calls == flock_calls


@patch("bijux_cli.services.history.fcntl")
def test_interprocess_lock_posix_exception(
    mock_fcntl: MagicMock, temp_history_file: Path
) -> None:
    """Test that the POSIX lock correctly handles exceptions during unlock."""
    mock_file = Mock()
    mock_file.fileno.return_value = 42
    mock_fcntl.flock.side_effect = [None, Exception("unlock fail")]
    with (
        patch("builtins.open", return_value=mock_file),
        _interprocess_lock(temp_history_file),
    ):
        pass


def test_maybe_simulate_disk_full() -> None:
    """Test that the disk full simulation is a no-op when the env var is not set."""
    _maybe_simulate_disk_full()


@patch.dict(os.environ, {"BIJUXCLI_TEST_DISK_FULL": "1"})
def test_maybe_simulate_disk_full_enabled() -> None:
    """Test that the disk full simulation raises an ENOSPC error when enabled."""
    with pytest.raises(OSError, match="No space left on device") as exc:
        _maybe_simulate_disk_full()
    assert exc.value.errno == errno.ENOSPC


def test_atomic_write_json(temp_history_file: Path) -> None:
    """Test that _atomic_write_json writes a list of events to a file."""
    events = [{"command": "test"}]
    _atomic_write_json(temp_history_file, events)
    assert json.loads(temp_history_file.read_text()) == events


def test_atomic_write_json_empty(temp_history_file: Path) -> None:
    """Test that _atomic_write_json correctly writes an empty list."""
    _atomic_write_json(temp_history_file, [])
    assert temp_history_file.read_text() == "[]\n"


def test_atomic_write_json_trim(temp_history_file: Path) -> None:
    """Test that _atomic_write_json trims the event list to the threshold."""
    events = [{"command": f"cmd{i}"} for i in range(_TRIM_THRESHOLD + 10)]
    _atomic_write_json(temp_history_file, events)
    written = json.loads(temp_history_file.read_text())
    assert len(written) == _TRIM_THRESHOLD
    assert written == events[-_TRIM_THRESHOLD:]


@patch.dict(os.environ, {"BIJUXCLI_TEST_DISK_FULL": "1"})
def test_atomic_write_json_disk_full(temp_history_file: Path) -> None:
    """Test that _atomic_write_json propagates a simulated disk full error."""
    with pytest.raises(OSError, match="No space") as exc:
        _atomic_write_json(temp_history_file, [])
    assert exc.value.errno == errno.ENOSPC


def test_atomic_write_json_permission_error(
    temp_history_file: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a PermissionError during directory creation is propagated."""

    def mock_mkdir(*args: Any, **kwargs: Any) -> None:
        raise PermissionError

    monkeypatch.setattr(Path, "mkdir", mock_mkdir)
    with pytest.raises(PermissionError):
        _atomic_write_json(temp_history_file, [])


def test_history_init(
    mock_telemetry: LoggingTelemetry, mock_observability: Observability
) -> None:
    """Test the initialization of the History service."""
    h = History(mock_telemetry, mock_observability)
    assert not h._events
    assert h._load_error is None


def test_get_history_path_default(history_no_path: History) -> None:
    """Test that the default history file path is used when none is provided."""
    with patch("bijux_cli.services.history.HISTORY_FILE", Path("/default/history")):
        assert history_no_path._get_history_path() == Path("/default/history")


def test_get_history_path_explicit(history: History) -> None:
    """Test that an explicitly provided history file path is used."""
    assert history._get_history_path() == history._explicit_path


@patch.dict(os.environ, {"BIJUXCLI_HISTORY_FILE": "/env/history"})
def test_get_history_path_env(history_no_path: History) -> None:
    """Test that the history path is correctly read from the environment variable."""
    assert history_no_path._get_history_path() == Path("/env/history")


@patch.dict(os.environ, {"BIJUXCLI_CONFIG": "/config/file"})
def test_get_history_path_from_config(history_no_path: History) -> None:
    """Test that the history path is derived from the config path if available."""
    assert history_no_path._get_history_path() == Path("/config/.bijux_history")


def test_reload_missing_file(history: History, temp_history_file: Path) -> None:
    """Test that reloading from a missing file results in an empty history."""
    temp_history_file.unlink(missing_ok=True)
    history._reload()
    assert not history._events
    assert history._load_error is None


def test_reload_empty_file(history: History, temp_history_file: Path) -> None:
    """Test that reloading from an empty file results in an empty history."""
    temp_history_file.write_text("")
    history._reload()
    assert not history._events
    assert history._load_error is None


def test_reload_non_array(history: History, temp_history_file: Path) -> None:
    """Test that reloading from a file with non-array JSON is handled."""
    temp_history_file.write_text('{"not": "array"}')
    history._reload()
    assert not history._events
    assert history._load_error is not None
    assert "not JSON array" in history._load_error


def test_reload_invalid_json(history: History, temp_history_file: Path) -> None:
    """Test that reloading from a file with invalid JSON is handled."""
    temp_history_file.write_text("invalid")
    history._reload()
    assert not history._events
    assert history._load_error


def test_reload_non_dict_entries(history: History, temp_history_file: Path) -> None:
    """Test that non-dictionary entries in the history file are skipped during reload."""
    temp_history_file.write_text('[1, {"command": "test"}, "str"]')
    history._reload()
    assert len(history._events) == 1
    assert history._events[0]["command"] == "test"


def test_reload_ascii_clean(history: History, temp_history_file: Path) -> None:
    """Test that command strings are cleaned to ASCII during reload."""
    temp_history_file.write_text('[{"command": "unicodé"}]')
    history._reload()
    assert history._events[0]["command"] == "unicode"


def test_reload_trim_in_memory(history: History, temp_history_file: Path) -> None:
    """Test that the in-memory event list is trimmed to the max size on reload."""
    events = json.dumps([{"command": f"cmd{i}"} for i in range(_MAX_IN_MEMORY + 10)])
    temp_history_file.write_text(events)
    history._reload()
    assert len(history._events) == _MAX_IN_MEMORY


def test_reload_exception(
    history: History, mock_observability: Observability, temp_history_file: Path
) -> None:
    """Test that a read exception during reload is handled gracefully."""
    with (
        patch("pathlib.Path.exists", return_value=True),
        patch("pathlib.Path.read_text", side_effect=Exception("read fail")),
    ):
        history._explicit_path = temp_history_file
        history._reload()
    assert not history._events
    assert history._load_error is not None
    assert "unreadable" in history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_dump(history: History, temp_history_file: Path) -> None:
    """Test that the _dump method correctly persists events to the file."""
    history._events = [{"command": "test"}]
    history._dump()
    assert json.loads(temp_history_file.read_text()) == [{"command": "test"}]


def test_dump_empty(history: History, temp_history_file: Path) -> None:
    """Test that _dump correctly writes an empty list."""
    history._events = []
    history._dump()
    assert temp_history_file.read_text() == "[]\n"


def test_dump_trim(history: History, temp_history_file: Path) -> None:
    """Test that _dump correctly trims the event list before writing."""
    history._events = [{"command": f"cmd{i}"} for i in range(_TRIM_THRESHOLD + 10)]
    history._dump()
    written = json.loads(temp_history_file.read_text())
    assert len(written) == _TRIM_THRESHOLD


def test_dump_permission_error(
    history: History, mock_observability: Observability, temp_history_file: Path
) -> None:
    """Test that a PermissionError during _dump is handled."""
    with (
        patch(
            "bijux_cli.services.history._atomic_write_json",
            side_effect=PermissionError("perm"),
        ),
        pytest.raises(PermissionError),
    ):
        history._dump()
    assert history._load_error is not None
    assert "write-permission" in history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_dump_enospc(
    history: History, mock_observability: Observability, temp_history_file: Path
) -> None:
    """Test that an ENOSPC error during _dump is handled."""
    exc = OSError(errno.ENOSPC, "full")
    with (
        patch("bijux_cli.services.history._atomic_write_json", side_effect=exc),
        pytest.raises(OSError, match="full"),
    ):
        history._dump()
    assert history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_dump_other_oserror(
    history: History, mock_observability: Observability, temp_history_file: Path
) -> None:
    """Test that a generic OSError during _dump is handled."""
    exc = OSError(errno.EIO, "io")
    with (
        patch("bijux_cli.services.history._atomic_write_json", side_effect=exc),
        pytest.raises(OSError, match="io"),
    ):
        history._dump()


def test_handle_dump_error(
    history: History,
    mock_observability: Observability,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test the internal error handling for dump operations."""
    exc = OSError("test exc")
    history._handle_dump_error("kind", exc, Path("/path"))
    assert history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]
    captured = capsys.readouterr()
    assert "kind" in captured.err


def test_add_basic(
    history: History, temp_history_file: Path, mock_telemetry: LoggingTelemetry
) -> None:
    """Test that the add method appends a correctly structured entry."""
    history.add("cmd", params=["arg"], success=True, return_code=0, duration_ms=100.5)
    assert len(history._events) == 1
    entry = history._events[0]
    assert entry["command"] == "cmd"
    assert entry["params"] == ["arg"]
    assert "timestamp" in entry
    assert entry["success"] is True
    assert entry["return_code"] == 0
    assert entry["duration_ms"] == 100.5
    assert json.loads(temp_history_file.read_text()) == [entry]
    mock_telemetry.event.assert_called_with(  # type: ignore[attr-defined]
        "history_event_added", {"command": "cmd"}
    )


def test_add_ascii_clean(history: History) -> None:
    """Test that the add method cleans the command string to ASCII."""
    history.add("unicodé")
    assert history._events[0]["command"] == "unicode"


def test_add_defaults(history: History) -> None:
    """Test that the add method uses correct default values for optional parameters."""
    history.add("cmd")
    entry = history._events[0]
    assert not entry["params"]
    assert entry["success"] is True
    assert entry["return_code"] == 0
    assert entry["duration_ms"] is None


def test_add_load_error(
    history: History,
    mock_observability: Observability,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test the add method's behavior when a load error is present."""

    def set_error() -> None:
        history._load_error = "load fail"

    with patch.object(history, "_reload", side_effect=set_error):
        history.add("cmd")
    mock_observability.log.assert_called()  # type: ignore[attr-defined]
    captured = capsys.readouterr()
    assert "load fail" in captured.err


def test_add_permission_error(
    history: History,
    mock_observability: Observability,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test that a PermissionError during add is handled."""
    with patch(
        "bijux_cli.services.history._atomic_write_json",
        side_effect=PermissionError("perm"),
    ):
        history.add("cmd")
    assert history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]
    captured = capsys.readouterr()
    assert "perm" in captured.err


def test_add_enospc(
    history: History,
    mock_observability: Observability,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test that an ENOSPC error during add is handled."""
    exc = OSError(errno.ENOSPC, "full")
    with patch("bijux_cli.services.history._atomic_write_json", side_effect=exc):
        history.add("cmd")
    assert history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]
    captured = capsys.readouterr()
    assert "full" in captured.err


def test_add_other_oserror(
    history: History,
    mock_observability: Observability,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Test that a generic OSError during add is handled."""
    exc = OSError(errno.EIO, "io")
    with patch("bijux_cli.services.history._atomic_write_json", side_effect=exc):
        history.add("cmd")
    assert history._load_error
    mock_observability.log.assert_called()  # type: ignore[attr-defined]
    captured = capsys.readouterr()
    assert "io" in captured.err


def test_add_telemetry_exception(
    history: History, mock_telemetry: LoggingTelemetry
) -> None:
    """Test that an exception from the telemetry service is handled gracefully."""
    mock_telemetry.event.side_effect = Exception("tel fail")  # type: ignore[attr-defined]
    history.add("cmd")


def test_list_basic(history: History) -> None:
    """Test that the list method returns all current entries."""
    history.add("cmd1")
    history.add("cmd2")
    assert len(history.list()) == 2


def test_list_limit(history: History) -> None:
    """Test that the list method respects the limit parameter."""
    for i in range(5):
        history.add(f"cmd{i}")
    assert len(history.list(limit=3)) == 3
    assert not history.list(limit=0)
    assert len(history.list(limit=None)) == 5


def test_list_group_by(history: History) -> None:
    """Test that the list method correctly groups entries by a specified key."""
    history.add("cmd", success=True)
    history.add("cmd", success=False)
    history.add("other", success=True)
    grouped = history.list(group_by="command")
    assert len(grouped) == 2
    assert any(g["group"] == "cmd" and g["count"] == 2 for g in grouped)
    assert any(g["group"] == "other" and g["count"] == 1 for g in grouped)


def test_list_group_by_missing(history: History) -> None:
    """Test that grouping falls back to 'unknown' if the key is missing."""
    history._events.append({"command": "cmd"})
    del history._events[0]["command"]
    with patch.object(history, "_reload", return_value=None):
        grouped = history.list(group_by="command")
    assert grouped[0]["group"] == "unknown"


def test_list_filter_cmd(history: History) -> None:
    """Test that the list method correctly filters entries by command."""
    history.add("abc")
    history.add("def")
    assert len(history.list(filter_cmd="bc")) == 1


def test_list_sort_timestamp(history: History) -> None:
    """Test that the list method correctly sorts entries by timestamp."""
    history._events.append({"command": "old", "timestamp": 1})
    history._events.append({"command": "new", "timestamp": 2})
    with patch.object(history, "_reload", return_value=None):
        entries = history.list(sort="timestamp")
    assert entries[0]["command"] == "old"


def test_list_dir_not_writable(
    history: History,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    mock_observability: Observability,
) -> None:
    """Test that a warning is logged if the history directory is not writable."""

    def mock_access(*args: Any) -> bool:
        return False

    monkeypatch.setattr(os, "access", mock_access)
    history.list()
    captured = capsys.readouterr()
    assert "Permission denied" in captured.err
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_list_load_error(history: History, temp_history_file: Path) -> None:
    """Test that the list method raises a RuntimeError if reloading fails."""
    history._explicit_path = temp_history_file
    with (
        patch("pathlib.Path.exists", return_value=True),
        patch("pathlib.Path.read_text", return_value="corrupt"),
        patch("json.loads", side_effect=Exception("corrupt file")),
        pytest.raises(RuntimeError, match="corrupt file"),
    ):
        history.list()


def test_clear(history: History, temp_history_file: Path) -> None:
    """Test that the clear method erases all history events."""
    history.add("cmd")
    history.clear()
    assert not history._events
    assert temp_history_file.read_text() == "[]\n"


def test_clear_exception(history: History, mock_observability: Observability) -> None:
    """Test that an exception during clear is propagated and logged."""
    with (
        patch(
            "bijux_cli.services.history._atomic_write_json",
            side_effect=RuntimeError("fail"),
        ),
        pytest.raises(RuntimeError, match="fail"),
    ):
        history.clear()
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_flush(history: History) -> None:
    """Test that the flush method calls the internal _dump method."""
    with patch.object(history, "_dump") as mock_dump:
        history.flush()
    mock_dump.assert_called_once()


def test_export(history: History, tmp_path: Path) -> None:
    """Test that the export method writes the current history to a file."""
    history.add("cmd")
    dest = tmp_path / "export.json"
    history.export(dest)
    assert json.loads(dest.read_text()) == history._events


def test_export_io_error(history: History) -> None:
    """Test that an error during export is wrapped in a RuntimeError."""
    with (
        patch.object(Path, "write_text", side_effect=Exception("io")),
        pytest.raises(RuntimeError),
    ):
        history.export(Path("/invalid"))


def test_import_(history: History, tmp_path: Path, temp_history_file: Path) -> None:
    """Test that the import_ method correctly appends entries from a file."""
    src = tmp_path / "import.json"
    src.write_text('[{"command": "imported"}]')
    history.import_(src)
    assert len(history._events) == 1
    assert history._events[0]["command"] == "imported"
    assert "timestamp" in history._events[0]
    assert json.loads(temp_history_file.read_text()) == history._events


def test_import_not_found(history: History) -> None:
    """Test that importing a non-existent file raises a RuntimeError."""
    with pytest.raises(RuntimeError, match="not found"):
        history.import_(Path("/nonexistent"))


def test_import_invalid_format(history: History, tmp_path: Path) -> None:
    """Test that importing a file with an invalid format raises a RuntimeError."""
    src = tmp_path / "invalid.json"
    src.write_text('{"not": "array"}')
    with pytest.raises(RuntimeError, match="Invalid import format"):
        history.import_(src)


def test_import_non_dict(history: History, tmp_path: Path) -> None:
    """Test that non-dictionary entries are skipped during import."""
    src = tmp_path / "import.json"
    src.write_text('[1, {"command": "ok"}, "str"]')
    history.import_(src)
    assert len(history._events) == 1
    assert history._events[0]["command"] == "ok"


def test_import_add_timestamp(history: History, tmp_path: Path) -> None:
    """Test that a timestamp is added to imported entries if missing."""
    src = tmp_path / "import.json"
    src.write_text('[{"command": "no ts"}]')
    history.import_(src)
    assert "timestamp" in history._events[0]


def test_import_trim(history: History, tmp_path: Path) -> None:
    """Test that the in-memory history is trimmed after a large import."""
    src = tmp_path / "import.json"
    large = [{"command": f"imp{i}"} for i in range(_MAX_IN_MEMORY + 10)]
    src.write_text(json.dumps(large))
    history.import_(src)
    assert len(history._events) == _MAX_IN_MEMORY


def test_import_load_error(history: History, tmp_path: Path) -> None:
    """Test that a load error during import is propagated."""

    def set_error() -> None:
        history._load_error = "load fail"

    with (
        patch.object(history, "_reload", side_effect=set_error),
        pytest.raises(RuntimeError, match="load fail"),
    ):
        history.import_(tmp_path / "src.json")


def test_import_exception(
    history: History, mock_observability: Observability, tmp_path: Path
) -> None:
    """Test that a generic exception during import is handled and logged."""
    src = tmp_path / "bad.json"
    src.write_text("bad")
    with pytest.raises(RuntimeError):
        history.import_(src)
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_import_telemetry(
    history: History, mock_telemetry: LoggingTelemetry, tmp_path: Path
) -> None:
    """Test that a telemetry event is emitted after a successful import."""
    src = tmp_path / "import.json"
    src.write_text('[{"command": "imp"}]')
    history.import_(src)
    mock_telemetry.event.assert_called_with("history_imported", {"count": 1})  # type: ignore[attr-defined]


def test_concurrent_add(history: History) -> None:
    """Test that adding entries concurrently from multiple threads is safe."""

    def worker() -> None:
        for i in range(10):
            history.add(f"cmd{i}")

    threads = [threading.Thread(target=worker) for _ in range(5)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    history._reload()
    assert len(history._events) == 50


def test_add_after_clear(history: History) -> None:
    """Test that adding an entry after clearing the history works correctly."""
    history.clear()
    history.add("new")
    assert len(history._events) == 1


def test_list_after_add(history: History) -> None:
    """Test that the list method reflects a newly added entry."""
    history.add("cmd")
    assert len(history.list()) == 1


def test_export_empty(history: History, tmp_path: Path) -> None:
    """Test that exporting an empty history results in an empty JSON array."""
    dest = tmp_path / "empty.json"
    history.export(dest)
    assert dest.read_text() == "[]\n"


def test_import_empty(history: History, tmp_path: Path) -> None:
    """Test that importing an empty JSON array results in an empty history."""
    src = tmp_path / "empty.json"
    src.write_text("[]")
    history.import_(src)
    assert not history._events


def test_import_ascii_clean(history: History, tmp_path: Path) -> None:
    """Test that commands are cleaned to ASCII during import."""
    src = tmp_path / "import.json"
    src.write_text('[{"command": "unicodé"}]')
    history.import_(src)
    assert history._events[0]["command"] == "unicode"


def test_list_group_by_limit_none(history: History) -> None:
    """Test that grouping works correctly when the limit is None."""
    history.add("cmd")
    assert len(history.list(group_by="command", limit=None)) == 1


def test_clear_telemetry(history: History, mock_telemetry: LoggingTelemetry) -> None:
    """Test that a telemetry event is emitted after clearing the history."""
    history.clear()
    mock_telemetry.event.assert_called_with("history_cleared", {})  # type: ignore[attr-defined]


def test_clear_load_error(history: History) -> None:
    """Test that clear works correctly even if a load error is present."""
    history._load_error = "error"
    history.clear()
    assert not history._events


def test_list_filter_case(history: History) -> None:
    """Test that command filtering is case-sensitive."""
    history.add("Cmd")
    assert not history.list(filter_cmd="cmd")


def test_group_unknown(history: History) -> None:
    """Test that a missing group key defaults to 'unknown'."""
    history._events.append({})
    with patch.object(history, "_reload", return_value=None):
        grouped = history.list(group_by="missing")
    assert grouped[0]["group"] == "unknown"


def test_list_limit_negative(history: History) -> None:
    """Test that a negative limit is treated as no limit."""
    history.add("cmd")
    assert len(history.list(limit=-1)) == 1


def test_reload_max_memory(history: History, temp_history_file: Path) -> None:
    """Test that reload trims the in-memory list to the maximum size."""
    events = [{"command": "cmd"} for _ in range(_MAX_IN_MEMORY + 1)]
    temp_history_file.write_text(json.dumps(events))
    history._reload()
    assert len(history._events) == _MAX_IN_MEMORY


def test_import_json_error(history: History, tmp_path: Path) -> None:
    """Test that a JSON decoding error during import is handled."""
    src = tmp_path / "bad.json"
    src.write_text("bad")
    with pytest.raises(RuntimeError):
        history.import_(src)


def test_add_no_params(history: History) -> None:
    """Test adding an event with None for the params argument."""
    history.add("cmd", params=None)
    assert not history._events[0]["params"]


def test_add_success_none(history: History) -> None:
    """Test adding an event with None for the success argument."""
    history.add("cmd", success=None)
    assert history._events[0]["success"] is False


def test_add_return_code_none(history: History) -> None:
    """Test adding an event with None for the return_code argument."""
    history.add("cmd", return_code=None)
    assert history._events[0]["return_code"] == 0


def test_add_duration_none(history: History) -> None:
    """Test adding an event with None for the duration_ms argument."""
    history.add("cmd", duration_ms=None)
    assert history._events[0]["duration_ms"] is None


def test_list_group_last_run(history: History) -> None:
    """Test that the 'last_run' field in grouped results is the max timestamp."""
    history._events.append({"command": "cmd", "timestamp": 1})
    history._events.append({"command": "cmd", "timestamp": 3})
    with patch.object(history, "_reload", return_value=None):
        grouped = history.list(group_by="command")
    assert grouped[0]["last_run"] == 3


def test_list_access_error(
    history: History,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    mock_observability: Observability,
) -> None:
    """Test that a permission error when checking the directory is handled."""

    def mock_access(*args: Any) -> bool:
        return False

    monkeypatch.setattr(os, "access", mock_access)
    history.list()
    captured = capsys.readouterr()
    assert "Permission denied" in captured.err
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_list_reload_sets_error(history: History) -> None:
    """Test that the list method raises a RuntimeError if reloading sets an error."""

    def set_error() -> None:
        history._load_error = "error"

    with (
        patch.object(history, "_reload", side_effect=set_error),
        pytest.raises(RuntimeError, match="error"),
    ):
        history.list()


def test_add_reload_error(
    history: History,
    capsys: pytest.CaptureFixture[str],
    mock_observability: Observability,
) -> None:
    """Test the add method's behavior when reloading fails."""

    def set_error() -> None:
        history._load_error = "reload fail"

    with patch.object(history, "_reload", side_effect=set_error):
        history.add("cmd")
    captured = capsys.readouterr()
    assert "reload fail" in captured.err
    mock_observability.log.assert_called()  # type: ignore[attr-defined]


def test_import_telemetry_exc(
    history: History, mock_telemetry: LoggingTelemetry, tmp_path: Path
) -> None:
    """Test that an exception from telemetry during import is handled gracefully."""
    mock_telemetry.event.side_effect = Exception("tel")  # type: ignore[attr-defined]
    src = tmp_path / "import.json"
    src.write_text("[]")
    history.import_(src)


def test_list_os_access_exception(history: History) -> None:
    """Test that an exception from os.access is swallowed during the list operation."""
    with patch("os.access", side_effect=RuntimeError("fail")):
        try:
            history.list()
        except Exception:
            pytest.fail("Exception should be swallowed in list()")


def test_import_adds_missing_timestamp(history: History, tmp_path: Path) -> None:
    """Test that a timestamp is added to imported events that are missing one."""
    data = [{"command": "foo"}]
    import_path = tmp_path / "import.json"
    import_path.write_text(json.dumps(data), encoding="utf-8")
    with patch("bijux_cli.services.history._now", return_value=123456.789):
        history.import_(import_path)
    events = history._events
    assert any(
        e.get("command") == "foo" and e.get("timestamp") == 123456.789 for e in events
    )


def test_import_preserves_existing_timestamp(history: History, tmp_path: Path) -> None:
    """Test that import_ preserves an existing timestamp and does not overwrite it."""
    import_file = tmp_path / "import_with_timestamp.json"
    original_timestamp = 123456789.0
    event_with_timestamp = [
        {"command": "command-with-timestamp", "timestamp": original_timestamp}
    ]
    import_file.write_text(json.dumps(event_with_timestamp), encoding="utf-8")

    with patch("bijux_cli.services.history._now") as mock_now:
        history.import_(import_file)
        mock_now.assert_not_called()

    history_file_path = history._get_history_path()
    final_history_data = json.loads(history_file_path.read_text(encoding="utf-8"))
    imported_event = next(
        (e for e in final_history_data if e["command"] == "command-with-timestamp"),
        None,
    )

    assert imported_event is not None
    assert imported_event["timestamp"] == original_timestamp
