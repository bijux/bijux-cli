# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the memory command."""

from __future__ import annotations

from collections.abc import Callable
import sys
from typing import Any
from unittest.mock import ANY, MagicMock, patch

from click import Command
from click.core import Context as ClickContext
import pytest
import typer
from typer import Context

from bijux_cli.commands.memory.clear import (
    _build_payload as clear_build_payload,  # pyright: ignore[reportPrivateUsage]
)
from bijux_cli.commands.memory.clear import clear_memory
from bijux_cli.commands.memory.delete import (
    _build_payload as delete_build_payload,  # pyright: ignore[reportPrivateUsage]
)
from bijux_cli.commands.memory.delete import delete_memory
from bijux_cli.commands.memory.get import (
    _build_payload as get_build_payload,  # pyright: ignore[reportPrivateUsage]
)
from bijux_cli.commands.memory.get import get_memory
from bijux_cli.commands.memory.list import (
    _build_payload as list_build_payload,  # pyright: ignore[reportPrivateUsage]
)
from bijux_cli.commands.memory.list import list_memory
from bijux_cli.commands.memory.service import (
    _build_payload as summary_build_payload,  # pyright: ignore[reportPrivateUsage]
)
from bijux_cli.commands.memory.service import (
    _run_one_shot_mode,  # pyright: ignore[reportPrivateUsage]
    memory,
    memory_summary,
)
from bijux_cli.commands.memory.set import (
    _build_payload as set_build_payload,  # pyright: ignore[reportPrivateUsage]
)
from bijux_cli.commands.memory.set import set_memory
from bijux_cli.commands.memory.utils import resolve_memory_service
from bijux_cli.core.enums import OutputFormat


@pytest.fixture
def mock_flags() -> dict[str, Any]:
    """Provide common CLI flags."""
    return {
        "quiet": False,
        "verbose": False,
        "fmt": "json",
        "pretty": True,
        "debug": False,
    }


@pytest.fixture
def mock_memory_svc() -> MagicMock:
    """Provide a MagicMock memory service."""
    return MagicMock()


def test_resolve_memory_service_success(mock_flags: dict[str, Any]) -> None:
    """Resolve memory service successfully."""
    with patch("bijux_cli.commands.memory.utils.DIContainer.current") as mock_current:
        mock_di_instance = MagicMock()
        mock_current.return_value = mock_di_instance
        mock_memory_svc = MagicMock()
        mock_di_instance.resolve.return_value = mock_memory_svc
        result = resolve_memory_service("command", "json", False, False, False)
        assert result == mock_memory_svc


def test_resolve_memory_service_exception(mock_flags: dict[str, Any]) -> None:
    """Emit error when memory service resolution fails."""
    with (
        patch("bijux_cli.commands.memory.utils.DIContainer.current") as mock_current,
        patch("bijux_cli.commands.memory.utils.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_di_instance = MagicMock()
        mock_current.return_value = mock_di_instance
        mock_di_instance.resolve.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            resolve_memory_service("command", "json", False, False, False)
        mock_emit.assert_called_with(
            "Memory service unavailable: error",
            code=1,
            failure="service_unavailable",
            command="command",
            fmt="json",
            quiet=False,
            include_runtime=False,
        )


def test_memory_summary_no_subcommand(mock_flags: dict[str, Any]) -> None:
    """Run one-shot summary when no subcommand invoked."""
    with (
        patch(
            "bijux_cli.commands.memory.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.memory.service.resolve_memory_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.memory.service._run_one_shot_mode") as mock_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.keys.return_value = ["key1"]
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        memory_summary(
            ctx,
            mock_flags["quiet"],
            mock_flags["verbose"],
            mock_flags["fmt"],
            mock_flags["pretty"],
            mock_flags["debug"],
        )
        mock_run.assert_called()


def test_memory_summary_keys_count_fail(mock_flags: dict[str, Any]) -> None:
    """Handle failure when counting keys in summary."""
    with (
        patch(
            "bijux_cli.commands.memory.service.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.memory.service.resolve_memory_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.memory.service._run_one_shot_mode") as mock_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.keys.side_effect = Exception("error")
        ctx = Context(MagicMock())
        ctx.invoked_subcommand = None
        memory_summary(
            ctx,
            mock_flags["quiet"],
            mock_flags["verbose"],
            mock_flags["fmt"],
            mock_flags["pretty"],
            mock_flags["debug"],
        )
        mock_run.assert_called_with(
            command="memory",
            fmt="json",
            output_format=OutputFormat.JSON,
            quiet=False,
            verbose=False,
            debug=False,
            effective_pretty=True,
            include_runtime=False,
            keys_count=None,
        )


def test_run_one_shot_mode(mock_flags: dict[str, Any]) -> None:
    """Emit payload in one-shot mode."""
    with (
        patch(
            "bijux_cli.commands.memory.service.contains_non_ascii_env",
            return_value=False,
        ),
        patch(
            "bijux_cli.commands.memory.service._build_payload",
            return_value={"status": "ok"},
        ),
        patch("bijux_cli.commands.memory.service.emit_and_exit") as mock_emit,
    ):
        _run_one_shot_mode(
            command="memory",
            fmt="json",
            output_format=OutputFormat.JSON,
            quiet=False,
            verbose=False,
            debug=False,
            effective_pretty=True,
            include_runtime=False,
            keys_count=0,
        )
        mock_emit.assert_called()


def test_run_one_shot_mode_ascii_env(mock_flags: dict[str, Any]) -> None:
    """Exit with error when environment is non-ASCII."""
    with (
        patch(
            "bijux_cli.commands.memory.service.contains_non_ascii_env",
            return_value=True,
        ),
        patch("bijux_cli.commands.memory.service.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            _run_one_shot_mode(
                command="memory",
                fmt="json",
                output_format=OutputFormat.JSON,
                quiet=False,
                verbose=False,
                debug=False,
                effective_pretty=True,
                include_runtime=False,
                keys_count=0,
            )
        mock_emit.assert_called_with(
            ANY,
            code=3,
            failure="ascii_env",
            command="memory",
            fmt="json",
            quiet=False,
            include_runtime=False,
        )


def test_run_one_shot_mode_value_error(mock_flags: dict[str, Any]) -> None:
    """Exit with error when payload builder raises ValueError."""
    with (
        patch(
            "bijux_cli.commands.memory.service.contains_non_ascii_env",
            return_value=False,
        ),
        patch(
            "bijux_cli.commands.memory.service._build_payload",
            side_effect=ValueError("error"),
        ),
        patch("bijux_cli.commands.memory.service.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            _run_one_shot_mode(
                command="memory",
                fmt="json",
                output_format=OutputFormat.JSON,
                quiet=False,
                verbose=False,
                debug=False,
                effective_pretty=True,
                include_runtime=False,
                keys_count=0,
            )
        mock_emit.assert_called_with(
            "error",
            code=3,
            failure="ascii",
            command="memory",
            fmt="json",
            quiet=False,
            include_runtime=False,
        )


def test_summary_build_payload() -> None:
    """Build summary payload."""
    payload = summary_build_payload(False, 5)
    assert payload["count"] == 5
    payload_verbose = summary_build_payload(True, 5)
    assert "python" in payload_verbose


def test_clear_memory_success(mock_flags: dict[str, Any]) -> None:
    """Clear memory successfully."""
    with (
        patch(
            "bijux_cli.commands.memory.clear.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.clear.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.clear.new_run_command") as mock_new_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        clear_memory(**mock_flags)
        mock_memory_svc.clear.assert_called_once()
        builder: Callable[[bool], dict[str, Any]] = mock_new_run.call_args.kwargs[
            "payload_builder"
        ]
        assert builder(False) == {"status": "cleared", "count": 0}
        assert "python" in builder(True)


def test_clear_memory_exception(mock_flags: dict[str, Any]) -> None:
    """Emit error when clearing memory fails."""
    with (
        patch(
            "bijux_cli.commands.memory.clear.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.clear.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.clear.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.clear.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            clear_memory(**mock_flags)
        mock_emit.assert_called_with(
            "Failed to clear memory: error",
            code=1,
            failure="clear_failed",
            command="memory clear",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_clear_build_payload() -> None:
    """Build clear payload."""
    payload = clear_build_payload(False)
    assert payload == {"status": "cleared", "count": 0}
    payload_verbose = clear_build_payload(True)
    assert "python" in payload_verbose


def test_delete_memory_success(mock_flags: dict[str, Any]) -> None:
    """Delete a memory key successfully."""
    with (
        patch(
            "bijux_cli.commands.memory.delete.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.memory.delete.resolve_memory_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.memory.delete.new_run_command") as mock_new_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        delete_memory("key", **mock_flags)
        mock_memory_svc.delete.assert_called_with("key")
        builder: Callable[[bool], dict[str, Any]] = mock_new_run.call_args.kwargs[
            "payload_builder"
        ]
        assert builder(False) == {"status": "deleted", "key": "key"}
        assert "python" in builder(True)


def test_delete_memory_invalid_key(mock_flags: dict[str, Any]) -> None:
    """Emit error when delete key is empty."""
    with (
        patch(
            "bijux_cli.commands.memory.delete.validate_common_flags",
            return_value="json",
        ),
        patch("bijux_cli.commands.memory.delete.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            delete_memory("", **mock_flags)
        mock_emit.assert_called()


def test_delete_memory_key_error(mock_flags: dict[str, Any]) -> None:
    """Emit not-found error when deleting unknown key."""
    with (
        patch(
            "bijux_cli.commands.memory.delete.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.memory.delete.resolve_memory_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.memory.delete.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.delete.side_effect = KeyError
        with pytest.raises(SystemExit):
            delete_memory("key", **mock_flags)
        mock_emit.assert_called_with(
            "Key not found: key",
            code=1,
            failure="not_found",
            command="memory delete",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_delete_memory_exception(mock_flags: dict[str, Any]) -> None:
    """Emit error when delete raises Exception."""
    with (
        patch(
            "bijux_cli.commands.memory.delete.validate_common_flags",
            return_value="json",
        ),
        patch(
            "bijux_cli.commands.memory.delete.resolve_memory_service"
        ) as mock_resolve,
        patch("bijux_cli.commands.memory.delete.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.delete.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            delete_memory("key", **mock_flags)
        mock_emit.assert_called_with(
            "Failed to delete memory key: error",
            code=1,
            failure="delete_failed",
            command="memory delete",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_delete_build_payload() -> None:
    """Build delete payload."""
    payload = delete_build_payload(False, "key")
    assert payload == {"status": "deleted", "key": "key"}
    payload_verbose = delete_build_payload(True, "key")
    assert "python" in payload_verbose


def test_get_memory_success(mock_flags: dict[str, Any]) -> None:
    """Get a memory key successfully."""
    with (
        patch(
            "bijux_cli.commands.memory.get.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.get.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.get.new_run_command") as mock_new_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.get.return_value = "value"
        get_memory("key", **mock_flags)
        mock_memory_svc.get.assert_called_with("key")
        builder: Callable[[bool], dict[str, Any]] = mock_new_run.call_args.kwargs[
            "payload_builder"
        ]
        assert builder(False) == {"status": "ok", "key": "key", "value": "value"}
        assert "python" in builder(True)


def test_get_memory_invalid_key(mock_flags: dict[str, Any]) -> None:
    """Emit error when get key is empty."""
    with (
        patch(
            "bijux_cli.commands.memory.get.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.get.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            get_memory("", **mock_flags)
        mock_emit.assert_called()


def test_get_memory_key_error(mock_flags: dict[str, Any]) -> None:
    """Emit not-found error when getting unknown key."""
    with (
        patch(
            "bijux_cli.commands.memory.get.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.get.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.get.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.get.side_effect = KeyError
        with pytest.raises(SystemExit):
            get_memory("key", **mock_flags)
        mock_emit.assert_called_with(
            "Key not found: key",
            code=1,
            failure="not_found",
            command="memory get",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_get_memory_exception(mock_flags: dict[str, Any]) -> None:
    """Emit error when get raises Exception."""
    with (
        patch(
            "bijux_cli.commands.memory.get.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.get.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.get.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.get.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            get_memory("key", **mock_flags)
        mock_emit.assert_called_with(
            "Failed to get memory: error",
            code=1,
            failure="get_failed",
            command="memory get",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_get_build_payload() -> None:
    """Build get payload."""
    payload = get_build_payload(False, "key", "value")
    assert payload == {"status": "ok", "key": "key", "value": "value"}
    payload_verbose = get_build_payload(True, "key", "value")
    assert "python" in payload_verbose


def test_list_memory_success(mock_flags: dict[str, Any]) -> None:
    """List memory keys successfully."""
    with (
        patch(
            "bijux_cli.commands.memory.list.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.list.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.list.new_run_command") as mock_new_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.keys.return_value = ["key1", "key2"]
        list_memory(**mock_flags)
        mock_memory_svc.keys.assert_called_once()
        builder: Callable[[bool], dict[str, Any]] = mock_new_run.call_args.kwargs[
            "payload_builder"
        ]
        assert builder(False) == {"status": "ok", "keys": ["key1", "key2"], "count": 2}
        assert "python" in builder(True)


def test_list_memory_exception(mock_flags: dict[str, Any]) -> None:
    """Emit error when list raises Exception."""
    with (
        patch(
            "bijux_cli.commands.memory.list.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.list.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.list.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.keys.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            list_memory(**mock_flags)
        mock_emit.assert_called_with(
            "Failed to list memory keys: error",
            code=1,
            failure="list_failed",
            command="memory list",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_list_build_payload() -> None:
    """Build list payload."""
    payload = list_build_payload(False, ["key1", "key2"])
    assert payload == {"status": "ok", "keys": ["key1", "key2"], "count": 2}
    payload_verbose = list_build_payload(True, ["key1", "key2"])
    assert "python" in payload_verbose


def test_set_memory_success(mock_flags: dict[str, Any]) -> None:
    """Set a memory key successfully."""
    with (
        patch(
            "bijux_cli.commands.memory.set.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.set.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.set.new_run_command") as mock_new_run,
    ):
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        set_memory("key", "value", **mock_flags)
        mock_memory_svc.set.assert_called_with("key", "value")
        builder: Callable[[bool], dict[str, Any]] = mock_new_run.call_args.kwargs[
            "payload_builder"
        ]
        assert builder(False) == {"status": "updated", "key": "key", "value": "value"}
        assert "python" in builder(True)


def test_set_memory_invalid_key(mock_flags: dict[str, Any]) -> None:
    """Emit error when set key is empty."""
    with (
        patch(
            "bijux_cli.commands.memory.set.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.set.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        with pytest.raises(SystemExit):
            set_memory("", "value", **mock_flags)
        mock_emit.assert_called()


def test_set_memory_exception(mock_flags: dict[str, Any]) -> None:
    """Emit error when set raises Exception."""
    with (
        patch(
            "bijux_cli.commands.memory.set.validate_common_flags", return_value="json"
        ),
        patch("bijux_cli.commands.memory.set.resolve_memory_service") as mock_resolve,
        patch("bijux_cli.commands.memory.set.emit_error_and_exit") as mock_emit,
    ):
        mock_emit.side_effect = SystemExit
        mock_memory_svc = MagicMock()
        mock_resolve.return_value = mock_memory_svc
        mock_memory_svc.set.side_effect = Exception("error")
        with pytest.raises(SystemExit):
            set_memory("key", "value", **mock_flags)
        mock_emit.assert_called_with(
            "Failed to set memory: error",
            code=1,
            failure="set_failed",
            command="memory set",
            fmt="json",
            quiet=False,
            include_runtime=False,
            debug=False,
        )


def test_set_build_payload() -> None:
    """Build set payload."""
    payload = set_build_payload(False, "key", "value")
    assert payload == {"status": "updated", "key": "key", "value": "value"}
    payload_verbose = set_build_payload(True, "key", "value")
    assert "python" in payload_verbose


class _DummyCmd(Command):
    """
    Base mock command that properly inherits from click.Command.
    This satisfies type checkers and provides a correct base for testing.
    """

    def __init__(self, name: str = "dummy", **kwargs: Any) -> None:
        super().__init__(name=name, **kwargs)
        self.allow_extra_args = False
        self.allow_interspersed_args = False
        self.ignore_unknown_options = False


class FakeCommandNoSub(_DummyCmd):
    """Dummy command that overrides get_help for top-level help tests."""

    def get_help(self, ctx: ClickContext) -> str:  # noqa: D401
        """Return specific help text for the test."""
        return "TOP HELP TEXT"


class FakeCommandWithSub(_DummyCmd):
    """Dummy command that provides a mock subcommand."""

    def get_command(self, ctx: ClickContext, name: str) -> Command | None:  # noqa: D401
        """Return a fake subcommand instance."""

        class Sub(_DummyCmd):
            """The mock subcommand, also a valid Command."""

            def get_help(self, ctx: ClickContext) -> str:  # noqa: D401
                """Return subcommand-specific help text."""
                return "SUBCOMMAND HELP TEXT"

        return Sub(name=name)


@pytest.mark.parametrize("help_arg", ["-h", "--help"])
def test_memory_help_no_subcommand_prints_top_help_and_exits(
    help_arg: str, capsys: pytest.CaptureFixture[str]
) -> None:
    """Print top help and exit when no subcommand."""
    old_argv = sys.argv[:]
    sys.argv[:] = ["prog", "memory", help_arg]
    ctx = Context(command=FakeCommandNoSub())
    ctx.invoked_subcommand = None
    with pytest.raises(typer.Exit):
        memory(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)
    out, err = capsys.readouterr()
    assert "TOP HELP TEXT" in out
    assert err == ""
    sys.argv[:] = old_argv


@pytest.mark.parametrize("help_arg", ["-h", "--help"])
def test_memory_help_with_subcommand_prints_sub_help_and_exits(
    help_arg: str, capsys: pytest.CaptureFixture[str]
) -> None:
    """Print subcommand help and exit when subcommand present."""
    old_argv = sys.argv[:]
    sys.argv[:] = ["prog", "memory", "anything", help_arg]
    ctx = Context(command=FakeCommandWithSub())
    ctx.invoked_subcommand = "anything"
    with pytest.raises(typer.Exit):
        memory(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)
    out, err = capsys.readouterr()
    assert "SUBCOMMAND HELP TEXT" in out
    assert err == ""
    sys.argv[:] = old_argv


class FakeCmd(_DummyCmd):
    """Another minimal command, now correctly inheriting from _DummyCmd."""

    pass


@pytest.mark.parametrize("help_arg", ["-h", "--help"])
def test_memory_help_no_subcommand_prints_top_help_and_exits_alt(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], help_arg: str
) -> None:
    """Print top help and exit (alternate path)."""
    monkeypatch.setattr(sys, "argv", ["prog", help_arg])
    monkeypatch.setattr(Context, "get_help", lambda self: "TOP HELP TEXT")
    ctx = Context(command=FakeCmd())
    ctx.invoked_subcommand = None
    with pytest.raises(typer.Exit):
        memory(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)
    out = capsys.readouterr().out
    assert "TOP HELP TEXT" in out


@pytest.mark.parametrize("help_arg", ["-h", "--help"])
def test_memory_help_with_subcommand_else_branch(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], help_arg: str
) -> None:
    """Print fallback help when subcommand not resolvable."""
    monkeypatch.setattr(sys, "argv", ["prog", help_arg])
    monkeypatch.setattr(Context, "get_help", lambda self: "SUBCMD BRANCH HELP")
    ctx = Context(command=FakeCmd())
    ctx.invoked_subcommand = "does_not_exist"
    with pytest.raises(typer.Exit):
        memory(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)
    out = capsys.readouterr().out
    assert "SUBCMD BRANCH HELP" in out


def test_memory_no_help_falls_through_to_summary(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fall through to memory_summary when help absent."""
    monkeypatch.setattr(sys, "argv", ["prog"])
    ctx = Context(command=FakeCmd())
    ctx.invoked_subcommand = None
    called: list[tuple[Context, bool, bool, str, bool, bool]] = []
    monkeypatch.setattr(
        "bijux_cli.commands.memory.service.memory_summary",
        lambda c, q, v, f, p, d: called.append((c, q, v, f, p, d)),
    )
    memory(ctx, quiet=True, verbose=True, fmt="yaml", pretty=False, debug=True)
    assert called == [(ctx, True, True, "yaml", False, True)]


def test_memory_fall_through_to_summary_and_exit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Exit propagated from memory_summary."""
    import bijux_cli.commands.memory.service as svc

    monkeypatch.setattr(sys, "argv", ["prog"])
    ctx = Context(command=FakeCmd())
    ctx.invoked_subcommand = None

    def fake_summary(*_args: Any, **_kwargs: Any) -> None:
        raise typer.Exit(code=42)

    monkeypatch.setattr(svc, "memory_summary", fake_summary)
    with pytest.raises(typer.Exit) as exc:
        svc.memory(
            ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False
        )
    assert exc.value.exit_code == 42


class MockTyperCommand(_DummyCmd):
    """Mock Typer command, now correctly inheriting from _DummyCmd."""

    pass


def test_memory_with_subcommand_does_not_call_summary() -> None:
    """Do nothing when subcommand is invoked."""
    with patch("bijux_cli.commands.memory.service.memory_summary") as mock_summary:
        ctx = Context(command=MockTyperCommand())
        ctx.invoked_subcommand = "set"
        with patch.object(sys, "argv", ["prog", "memory", "set"]):
            memory(
                ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False
            )
    mock_summary.assert_not_called()
