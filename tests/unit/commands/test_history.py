# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the history command."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any
from unittest.mock import ANY, MagicMock, patch

import pytest
from typer import Context

from bijux_cli.commands.history.clear import clear_history
from bijux_cli.commands.history.clear import resolve_history_service as clear_resolve
from bijux_cli.commands.history.service import history, resolve_history_service


@pytest.fixture
def mock_flags() -> dict[str, Any]:
    """Return mock common flags."""
    return {
        "quiet": False,
        "verbose": False,
        "fmt": "json",
        "pretty": True,
        "debug": False,
    }


@pytest.fixture
def mock_history_svc() -> MagicMock:
    """Return a mock history service."""
    mock = MagicMock()
    return mock


def test_resolve_history_service_success(mock_flags: dict[str, Any]) -> None:
    """Test successful resolution of the history service."""
    with patch(
        "bijux_cli.commands.history.service.DIContainer.current"
    ) as mock_current:
        mock_di_instance = MagicMock()
        mock_current.return_value = mock_di_instance
        mock_history_svc = MagicMock()
        mock_di_instance.resolve.return_value = mock_history_svc
        result = resolve_history_service("command", "json", False, False, False)
        assert result == mock_history_svc


def test_resolve_history_service_exception(mock_flags: dict[str, Any]) -> None:
    """Test exception handling during history service resolution."""
    with patch(
        "bijux_cli.commands.history.service.DIContainer.current"
    ) as mock_current:
        mock_di_instance = MagicMock()
        mock_current.return_value = mock_di_instance
        mock_di_instance.resolve.side_effect = Exception("error")
        with patch(
            "bijux_cli.commands.history.service.emit_error_and_exit"
        ) as mock_emit:
            mock_emit.side_effect = SystemExit
            with pytest.raises(SystemExit):
                resolve_history_service("command", "json", False, False, False)
            mock_emit.assert_called_with(
                "History service unavailable: error",
                code=1,
                failure="service_unavailable",
                command="command",
                fmt="json",
                quiet=False,
                include_runtime=False,
                debug=False,
            )


def test_history_no_subcommand(mock_flags: dict[str, Any]) -> None:
    """Test the history command with no subcommand provided."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.return_value = [{"command": "cmd1"}]

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(ctx, 20, None, None, None, None, None, **mock_flags)  # type: ignore[arg-type]
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"entries": [{"command": "cmd1"}]}
        assert "python" in builder(True)


def test_history_limit_zero(mock_flags: dict[str, Any]) -> None:
    """Test the history command with a limit of zero."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.return_value = [{"command": "cmd1"}]
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(ctx, 0, None, None, None, None, None, **mock_flags)  # type: ignore[arg-type]
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"entries": []}


def test_history_filter_cmd(mock_flags: dict[str, Any]) -> None:
    """Test the history command with a command filter."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.return_value = [
            {"command": "cmd1"},
            {"command": "cmd2"},
        ]
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(ctx, 20, None, "cmd1", None, None, None, **mock_flags)  # type: ignore[arg-type]
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"entries": [{"command": "cmd1"}]}


def test_history_sort_timestamp(mock_flags: dict[str, Any]) -> None:
    """Test the history command with sorting by timestamp."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.return_value = [{"timestamp": 2}, {"timestamp": 1}]
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(ctx, 20, None, None, "timestamp", None, None, **mock_flags)  # type: ignore[arg-type]
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"entries": [{"timestamp": 1}, {"timestamp": 2}]}


def test_history_group_by_command(mock_flags: dict[str, Any]) -> None:
    """Test the history command with grouping by command."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.return_value = [
            {"command": "cmd1"},
            {"command": "cmd1"},
            {"command": "cmd2"},
        ]
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(ctx, 20, "command", None, None, None, None, **mock_flags)  # type: ignore[arg-type]
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        entries = builder(False)["entries"]
        assert len(entries) == 2
        assert any(e["group"] == "cmd1" and e["count"] == 2 for e in entries)
        assert any(e["group"] == "cmd2" and e["count"] == 1 for e in entries)


def test_history_export_path(mock_flags: dict[str, Any]) -> None:
    """Test the history command with the export option."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.Path") as mock_path,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.return_value = [{"command": "cmd1"}]

        mock_file = MagicMock()
        mock_path.return_value = mock_file

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            20,
            None,
            None,
            None,
            "export.json",
            None,  # type: ignore[arg-type]
            **mock_flags,
        )

        mock_file.write_text.assert_called()

        first_call_kwargs = mock_new_run.call_args_list[0].kwargs
        builder = first_call_kwargs["payload_builder"]
        assert builder(False) == {"status": "exported", "file": "export.json"}


def test_history_import_path(mock_flags: dict[str, Any]) -> None:
    """Test the history command with the import option."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.Path") as mock_path,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_file = MagicMock()
        mock_path.return_value = mock_file
        mock_file.read_text.return_value = json.dumps(
            [
                {
                    "command": "cmd1",
                    "params": [],
                    "success": True,
                    "return_code": 0,
                    "duration_ms": 0.0,
                }
            ]
        )
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            20,
            None,
            None,
            None,
            None,  # type: ignore[arg-type]
            "import.json",
            **mock_flags,
        )

        mock_history_svc.clear.assert_called_once()
        mock_history_svc.add.assert_called_with(
            command="cmd1",
            params=[],
            success=True,
            return_code=0,
            duration_ms=0.0,
        )

        first_call_kwargs = mock_new_run.call_args_list[0].kwargs
        builder = first_call_kwargs["payload_builder"]
        assert builder(False) == {"status": "imported", "file": "import.json"}


def test_history_invalid_limit(mock_flags: dict[str, Any]) -> None:
    """Error when --limit is negative."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=MagicMock(),
        ),
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(ctx, -1, None, None, None, None, None, **mock_flags)  # type: ignore[arg-type]

        mock_emit.assert_called_with(
            "Invalid value for --limit: must be non-negative.",
            code=2,
            failure="limit",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_invalid_sort(mock_flags: dict[str, Any]) -> None:
    """Error when --sort uses an unsupported key."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=MagicMock(),
        ),
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(ctx, 20, None, None, "invalid", None, None, **mock_flags)  # type: ignore[arg-type]

        mock_emit.assert_called_with(
            "Invalid sort key: only 'timestamp' is supported.",
            code=2,
            failure="sort",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_invalid_group_by(mock_flags: dict[str, Any]) -> None:
    """Error when --group-by uses an unsupported key."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=MagicMock(),
        ),
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(ctx, 20, "invalid", None, None, None, None, **mock_flags)  # type: ignore[arg-type]

        mock_emit.assert_called_with(
            "Invalid group_by: only 'command' is supported.",
            code=2,
            failure="group_by",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_import_invalid_json(mock_flags: dict[str, Any]) -> None:
    """Error when importing a file with invalid JSON content."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=MagicMock(),
        ),
        patch("bijux_cli.commands.history.service.Path") as mock_path,
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        mock_file = MagicMock()
        mock_path.return_value = mock_file
        mock_file.read_text.return_value = "invalid"
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(
                ctx,
                20,
                None,
                None,
                None,
                None,  # type: ignore[arg-type]
                "import.json",
                **mock_flags,
            )

        mock_emit.assert_called_with(
            ANY,
            code=2,
            failure="import_failed",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_import_not_list(mock_flags: dict[str, Any]) -> None:
    """Error when imported JSON root is not a list."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=MagicMock(),
        ),
        patch("bijux_cli.commands.history.service.Path") as mock_path,
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        mock_file = MagicMock()
        mock_path.return_value = mock_file
        mock_file.read_text.return_value = json.dumps({})
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(ctx, 20, None, None, None, None, "import.json", **mock_flags)  # type: ignore[arg-type]

        mock_emit.assert_called_with(
            ANY,
            code=2,
            failure="import_failed",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_import_non_dict(mock_flags: dict[str, Any]) -> None:
    """Test importing a list containing non-dictionary items."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.Path") as mock_path,
        patch("bijux_cli.commands.history.service.new_run_command") as _,
        patch("typer.Exit") as _,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_file = MagicMock()
        mock_path.return_value = mock_file
        mock_file.read_text.return_value = json.dumps([1])
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            20,
            None,
            None,
            None,
            None,  # type: ignore[arg-type]
            "import.json",
            **mock_flags,
        )
        mock_history_svc.clear.assert_called_once()
        mock_history_svc.add.assert_not_called()


def test_history_export_exception(mock_flags: dict[str, Any]) -> None:
    """Test exception handling during history export."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.list.side_effect = Exception("error")
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(ctx, 20, None, None, None, "export.json", None, **mock_flags)  # type: ignore[arg-type]

        mock_emit.assert_called_with(
            ANY,
            code=2,
            failure="export_failed",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_clear_history_success(mock_flags: dict[str, Any]) -> None:
    """Test successful clearing of the history."""
    with (
        patch(
            "bijux_cli.commands.history.clear.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.clear.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.clear.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        clear_history(**mock_flags)
        mock_history_svc.clear.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        assert builder(False) == {"status": "cleared"}
        assert "python" in builder(True)


def test_clear_history_exception(mock_flags: dict[str, Any]) -> None:
    """Test exception handling when clearing history fails."""
    with (
        patch(
            "bijux_cli.commands.history.clear.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.clear.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.clear.emit_error_and_exit") as mock_emit,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        mock_history_svc.clear.side_effect = Exception("error")
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            clear_history(**mock_flags)
        mock_emit.assert_called_with(
            "Failed to clear history: error",
            code=1,
            failure="clear_failed",
            command="history clear",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_clear_resolve_history_service_exception(mock_flags: dict[str, Any]) -> None:
    """Test exception handling during history service resolution for the clear command."""
    with (
        patch("bijux_cli.commands.history.clear.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.history.clear.emit_error_and_exit") as mock_emit,
    ):
        mock_di_instance = MagicMock()
        mock_current.return_value = mock_di_instance
        mock_di_instance.resolve.side_effect = Exception("error")
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            clear_resolve("command", "json", False, False, False)
        mock_emit.assert_called_with(
            "History service unavailable: error",
            code=1,
            failure="service_unavailable",
            command="command",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_clear_history_debug_overrides_flags(mock_flags: dict[str, Any]) -> None:
    """Test that the debug flag correctly overrides other flags."""
    flags = {**mock_flags, "debug": True}
    with (
        patch(
            "bijux_cli.commands.history.clear.validate_common_flags",
            return_value="json",
        ) as _,
        patch(
            "bijux_cli.commands.history.clear.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.clear.new_run_command") as mock_new_run,
    ):
        mock_history_svc = MagicMock()
        mock_resolve.return_value = mock_history_svc
        clear_history(**flags)  # pyright: ignore[reportArgumentType]

        call_kwargs = mock_new_run.call_args.kwargs
        assert call_kwargs["verbose"] is True
        assert call_kwargs["pretty"] is True

        builder = call_kwargs["payload_builder"]
        payload = builder(True)
        assert payload["status"] == "cleared"
        assert "python" in payload
        assert "platform" in payload


def test_resolve_history_service_error(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test the exception path in resolve_history_service."""
    fake = MagicMock()
    fake.resolve.side_effect = RuntimeError("boom")
    with (
        patch(
            "bijux_cli.commands.history.service.DIContainer.current", return_value=fake
        ) as _,
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as err,
    ):
        err.side_effect = SystemExit()
        with pytest.raises(SystemExit):
            resolve_history_service("history", "json", False, False, False)
        err.assert_called_once_with(
            "History service unavailable: boom",
            code=1,
            failure="service_unavailable",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_list_positive_limit_and_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test limit slicing and the list_failed exception path."""
    entries = [{"command": "x"}, {"command": "y"}, {"command": "z"}]
    svc = MagicMock()
    svc.list.return_value = entries
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        history(
            Context(MagicMock()),
            2,
            None,
            None,
            None,
            None,  # type: ignore[arg-type]
            None,  # type: ignore[arg-type]
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )
        builder = new_run.call_args.kwargs["payload_builder"]
        assert builder(False)["entries"] == entries[-2:]

    svc2 = MagicMock()
    svc2.list.side_effect = ValueError("nope")
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc2,
        ),
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as err,
    ):
        err.side_effect = SystemExit()
        with pytest.raises(SystemExit):
            history(
                Context(MagicMock()),
                2,
                None,
                None,
                None,
                None,  # type: ignore[arg-type]
                None,  # type: ignore[arg-type]
                quiet=False,
                verbose=False,
                fmt="json",
                pretty=False,
                debug=False,
            )
        err.assert_called_once_with(
            "Failed to list history: nope",
            code=1,
            failure="list_failed",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_export_payload_and_runtime(tmp_path: Path) -> None:
    """Test that verbose export includes runtime metadata in the payload."""
    out_path = tmp_path / "out.json"
    svc = MagicMock()
    svc.list.return_value = [{"a": 1}]
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch.object(Path, "write_text") as write_text,
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            limit=20,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=str(out_path),
            import_path=None,  # type: ignore[arg-type]
            quiet=False,
            verbose=True,
            fmt="json",
            pretty=False,
            debug=False,
        )

    write_text.assert_called()
    first_kwargs = new_run.call_args_list[0].kwargs
    builder = first_kwargs["payload_builder"]
    payload = builder(True)
    assert payload["status"] == "exported"
    assert payload["file"] == str(out_path)
    assert "python" in payload
    assert "platform" in payload


def test_history_debug_flag_overrides() -> None:
    """Test that debug=True forces verbose=True and pretty=True."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags"
        ) as mock_validate,
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        mock_validate.return_value = "json"
        mock_resolve.return_value = MagicMock()
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        history(
            ctx,
            limit=5,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=None,  # type: ignore[arg-type]
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=True,
        )

        mock_validate.assert_called_once_with(
            "json", "history", False, include_runtime=True
        )

        _, kwargs = mock_new_run.call_args
        assert kwargs["verbose"] is True
        assert kwargs["pretty"] is True
        assert kwargs["debug"] is True


def test_history_invoked_subcommand_skips() -> None:
    """Test that history() returns early if a subcommand was invoked."""
    with patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run:
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = "some_sub"
        history(
            ctx,
            10,
            None,
            None,
            None,
            None,  # type: ignore[arg-type]
            None,  # type: ignore[arg-type]
            False,
            False,
            "json",
            True,
            False,
        )
        mock_new_run.assert_not_called()


def test_history_list_failure() -> None:
    """Test the structured exit on history list failure."""
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.history.service.emit_error_and_exit") as mock_emit,
    ):
        svc = MagicMock()
        svc.list.side_effect = Exception("boom")
        mock_resolve.return_value = svc
        mock_emit.side_effect = SystemExit

        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        with pytest.raises(SystemExit):
            history(
                ctx,
                5,
                None,
                None,
                None,
                None,  # type: ignore[arg-type]
                None,  # type: ignore[arg-type]
                False,
                False,
                "json",
                True,
                False,
            )

        mock_emit.assert_called_once_with(
            "Failed to list history: boom",
            code=1,
            failure="list_failed",
            command="history",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_history_import_skip_empty_and_payload(tmp_path: Path) -> None:
    """Test that empty commands are skipped on import and check the payload."""
    data = [
        {"cmd": ""},
        {
            "command": "good",
            "params": [],
            "success": True,
            "return_code": 0,
            "duration_ms": 1.23,
        },
    ]
    p = tmp_path / "inp.json"
    p.write_text(json.dumps(data))
    svc = MagicMock()

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch.object(Path, "read_text", return_value=p.read_text()),
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            limit=20,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=str(p),
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )

    svc.clear.assert_called_once()
    svc.add.assert_called_once_with(
        command="good", params=[], success=True, return_code=0, duration_ms=1.23
    )

    first_kwargs = new_run.call_args_list[0].kwargs
    builder = first_kwargs["payload_builder"]
    payload = builder(False)
    assert payload["status"] == "imported"
    assert payload["file"].endswith("inp.json")


def test_history_import_payload_runtime_with_verbose(tmp_path: Path) -> None:
    """Test that verbose import includes runtime metadata in the payload."""
    data = [{"command": "cmd"}]
    p = tmp_path / "in2.json"
    p.write_text(json.dumps(data))
    svc = MagicMock()

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch.object(Path, "read_text", return_value=p.read_text()),
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            limit=20,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=str(p),
            quiet=False,
            verbose=True,
            fmt="json",
            pretty=False,
            debug=False,
        )

    first_kwargs = new_run.call_args_list[0].kwargs
    builder = first_kwargs["payload_builder"]
    payload = builder(True)
    assert payload["status"] == "imported"
    assert payload["file"].endswith("in2.json")
    assert "python" in payload
    assert "platform" in payload


def test_history_export_payload_and_basic(tmp_path: Path) -> None:
    """Test the basic export payload without runtime metadata."""
    out = tmp_path / "out.json"
    svc = MagicMock(list=MagicMock(return_value=[{"foo": "bar"}]))

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch.object(Path, "write_text") as write_text,
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            limit=20,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=str(out),
            import_path=None,  # type: ignore[arg-type]
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )

    write_text.assert_called_once()
    first_kwargs = new_run.call_args_list[0].kwargs
    builder = first_kwargs["payload_builder"]
    payload = builder(False)
    assert payload["status"] == "exported"
    assert payload["file"] == str(out)


def test_history_export_payload_runtime_with_verbose(tmp_path: Path) -> None:
    """Test that verbose export includes runtime metadata in the payload."""
    out = tmp_path / "out2.json"
    svc = MagicMock(list=MagicMock(return_value=[{"foo": "bar"}]))

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch.object(Path, "write_text") as write_text,
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            limit=20,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=str(out),
            import_path=None,  # type: ignore[arg-type]
            quiet=False,
            verbose=True,
            fmt="json",
            pretty=False,
            debug=False,
        )

    write_text.assert_called_once()
    first_kwargs = new_run.call_args_list[0].kwargs
    builder = first_kwargs["payload_builder"]
    payload = builder(True)
    assert payload["status"] == "exported"
    assert payload["file"] == str(out)
    assert "python" in payload
    assert "platform" in payload


def test_history_list_limit_slicing(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that a positive limit correctly slices the last N entries."""
    svc = MagicMock(list=MagicMock(return_value=[{"id": 1}, {"id": 2}, {"id": 3}]))
    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        history(
            ctx,
            limit=2,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=None,  # type: ignore[arg-type]
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )

    last_kwargs = new_run.call_args_list[-1].kwargs
    builder = last_kwargs["payload_builder"]
    payload = builder(False)
    assert payload["entries"] == [{"id": 2}, {"id": 3}]


def test_history_limit_positive_slicing(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test slicing with a positive limit and payload builder with/without runtime."""
    entries = [{"command": "one"}, {"command": "two"}, {"command": "three"}]
    svc = MagicMock(list=MagicMock(return_value=entries))

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=svc,
        ),
        patch("bijux_cli.commands.history.service.new_run_command") as new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        history(
            ctx,
            limit=2,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=None,  # type: ignore[arg-type]
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=False,
            debug=False,
        )

    builder = new_run.call_args.kwargs["payload_builder"]

    payload_no_rt = builder(False)
    assert payload_no_rt["entries"] == entries[-2:]

    payload_rt = builder(True)
    assert payload_rt["entries"] == entries[-2:]
    assert "python" in payload_rt
    assert "platform" in payload_rt


def test_history_positive_limit_branch_and_payload_builder(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test the positive limit branch and the final payload builder."""
    fake_svc = MagicMock()
    fake_svc.list.return_value = [
        {"command": "one", "timestamp": 100},
        {"command": "two", "timestamp": 200},
        {"command": "three", "timestamp": 300},
    ]

    monkeypatch.setenv("BIJUXCLI_HISTORY_LIMIT", "")
    monkeypatch.setattr(
        "bijux_cli.commands.history.service.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.history.service.resolve_history_service",
        lambda *args, **kwargs: fake_svc,
    )

    captured = {}
    monkeypatch.setattr(
        "bijux_cli.commands.history.service.new_run_command",
        lambda **kwargs: captured.update(kwargs),
    )

    ctx = Context(MagicMock())
    ctx.invoked_subcommand = None

    history(
        ctx,
        limit=2,
        group_by=None,
        filter_cmd=None,
        sort=None,
        export_path=None,  # type: ignore[arg-type]
        import_path=None,  # type: ignore[arg-type]
        quiet=False,
        verbose=False,
        fmt="json",
        pretty=False,
        debug=False,
    )

    assert "payload_builder" in captured

    builder = captured["payload_builder"]

    payload = builder(False)
    assert payload["entries"] == [
        {"command": "two", "timestamp": 200},
        {"command": "three", "timestamp": 300},
    ]

    payload_rt = builder(True)
    assert payload_rt["entries"] == payload["entries"]
    assert "python" in payload_rt
    assert "platform" in payload_rt


def test_history_list_slicing_for_positive_limit(
    mock_history_svc: MagicMock, mock_flags: dict[str, Any]
) -> None:
    """Test the successful execution path when limit is greater than 0."""
    all_entries = [
        {"command": "cmd1", "timestamp": 1},
        {"command": "cmd2", "timestamp": 2},
        {"command": "cmd3", "timestamp": 3},
    ]
    mock_history_svc.list.return_value = all_entries

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=mock_history_svc,
        ),
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        history(
            ctx,
            limit=2,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=None,  # type: ignore[arg-type]
            **mock_flags,
        )

        mock_new_run.assert_called_once()
        builder = mock_new_run.call_args.kwargs["payload_builder"]
        payload = builder(False)

        assert payload["entries"] == [
            {"command": "cmd2", "timestamp": 2},
            {"command": "cmd3", "timestamp": 3},
        ]


def test_history_positive_limit_slicing_and_successful_completion(
    mock_history_svc: MagicMock, mock_flags: dict[str, Any]
) -> None:
    """Test successful slicing and payload creation for a positive limit."""
    history_entries = [
        {"command": "cmd1", "timestamp": 1660000001},
        {"command": "cmd2", "timestamp": 1660000002},
        {"command": "cmd3", "timestamp": 1660000003},
    ]
    mock_history_svc.list.return_value = history_entries

    with (
        patch(
            "bijux_cli.commands.history.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.history.service.resolve_history_service",
            return_value=mock_history_svc,
        ),
        patch("bijux_cli.commands.history.service.new_run_command") as mock_new_run,
    ):
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None

        history(
            ctx,
            limit=2,
            group_by=None,
            filter_cmd=None,
            sort=None,
            export_path=None,  # type: ignore[arg-type]
            import_path=None,  # type: ignore[arg-type]
            **mock_flags,
        )

        mock_new_run.assert_called_once()
        payload_builder = mock_new_run.call_args.kwargs["payload_builder"]
        result_payload = payload_builder(False)

        assert "entries" in result_payload
        assert len(result_payload["entries"]) == 2
        assert result_payload["entries"] == history_entries[-2:]
