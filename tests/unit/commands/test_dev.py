# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the dev command."""

# pyright: reportPrivateUsage=false
# pyright: reportMissingParameterType=false
from __future__ import annotations

import json
from pathlib import Path
from typing import Any, cast
from unittest.mock import MagicMock

import pytest
import typer
from typer import Context
import yaml

import bijux_cli.commands.dev as dev_pkg
from bijux_cli.commands.dev.di import _build_dev_di_payload, _key_to_name, dev_di_graph
from bijux_cli.commands.dev.list_plugins import dev_list_plugins
from bijux_cli.commands.dev.service import dev


@pytest.fixture
def ctx() -> Context:
    """Provide a default Typer Context with a mocked command."""
    return Context(MagicMock())


def test_dev_package_import_and_app_wiring() -> None:
    """Test that the dev Typer app and its commands are registered correctly."""
    assert hasattr(dev_pkg, "dev_app")
    app = dev_pkg.dev_app
    assert app.info.help == "Developer tools and diagnostics."

    names = {cmd.name for cmd in app.registered_commands}
    assert {"di", "list-plugins"}.issubset(names)


def test_key_to_name_with_string() -> None:
    """Test that a string key is returned as is."""
    assert _key_to_name("proto") == "proto"


def test_key_to_name_with_class() -> None:
    """Test that a class key is converted to its name."""

    class MyProto: ...

    assert _key_to_name(MyProto) == "MyProto"


def test_key_to_name_with_object_without_name() -> None:
    """Test that an object key falls back to its string representation."""
    obj = object()
    assert _key_to_name(obj) == str(obj)


def test_build_dev_di_payload_without_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test building the DI payload without runtime information."""
    mock_di = MagicMock()
    mock_di.factories.return_value = [(str, "E"), ("X", "Y")]
    mock_di.services.return_value = [(int, "I"), ("S", "T")]
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.DIContainer.current",
        lambda: mock_di,
    )

    payload = _build_dev_di_payload(include_runtime=False)
    assert payload["factories"] == [
        {"protocol": "str", "alias": "E"},
        {"protocol": "X", "alias": "Y"},
    ]
    assert payload["services"] == [
        {"protocol": "int", "alias": "I", "implementation": None},
        {"protocol": "S", "alias": "T", "implementation": None},
    ]
    assert "python" not in payload
    assert "platform" not in payload


def test_build_dev_di_payload_with_runtime(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test building the DI payload with runtime information."""
    mock_di = MagicMock()
    mock_di.factories.return_value = []
    mock_di.services.return_value = []
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.DIContainer.current",
        lambda: mock_di,
    )

    payload = _build_dev_di_payload(include_runtime=True)
    assert payload["factories"] == []
    assert payload["services"] == []
    assert "python" in payload
    assert "platform" in payload


def test_dev_di_graph_basic_json_calls_new_run_command(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that the di-graph command calls new_run_command with the correct payload."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )

    base_payload = {"factories": [{"protocol": "A", "alias": "a"}], "services": []}
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: dict(base_payload),
    )

    captured: dict[str, Any] = {}

    def fake_new_run_command(**kwargs: Any) -> None:
        captured["kwargs"] = kwargs
        built = kwargs["payload_builder"](False)
        captured["built"] = built

    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.new_run_command",
        fake_new_run_command,
    )

    dev_di_graph(
        quiet=False,
        verbose=False,
        fmt="json",
        pretty=True,
        debug=False,
        output=[],
    )
    assert captured["built"] == base_payload
    assert captured["kwargs"]["fmt"] == "json"
    assert not captured["kwargs"]["verbose"]


def test_dev_di_graph_limit_env_trims_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that BIJUXCLI_DI_LIMIT trims the DI graph payload."""
    monkeypatch.setenv("BIJUXCLI_DI_LIMIT", "1")

    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )

    payload_in: dict[str, Any] = {
        "factories": [{"protocol": "A", "alias": "a"}, {"protocol": "B", "alias": "b"}],
        "services": [
            {"protocol": "C", "alias": "c", "implementation": None},
            {"protocol": "D", "alias": "d", "implementation": None},
        ],
    }
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: dict(payload_in),
    )

    out: dict[str, Any] = {}

    def _fake_new_run_command(**kw: Any) -> None:
        payload = cast(dict[str, Any], kw["payload_builder"](False))
        out["payload"] = payload

    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.new_run_command", _fake_new_run_command
    )

    dev_di_graph(
        quiet=False,
        verbose=False,
        fmt="json",
        pretty=True,
        debug=False,
        output=[],
    )

    payload_out = cast(dict[str, Any], out["payload"])
    assert payload_out["factories"] == [payload_in["factories"][0]]
    assert payload_out["services"] == [payload_in["services"][0]]


def test_dev_di_graph_output_json_writes_file_and_calls_new_run(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that JSON output is written to a file and new_run_command is called."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )

    payload_in = {
        "factories": [],
        "services": [{"protocol": "X", "alias": "x", "implementation": None}],
    }
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: dict(payload_in),
    )

    called: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.new_run_command",
        lambda **kw: called.update({"built": kw["payload_builder"](False)}),
    )

    out_path = tmp_path / "di.json"
    dev_di_graph(
        quiet=False,
        verbose=False,
        fmt="json",
        pretty=True,
        debug=False,
        output=[out_path],
    )

    data = json.loads(out_path.read_text("utf-8"))
    assert data == payload_in
    assert called["built"] == payload_in


def test_dev_di_graph_output_yaml_writes_file(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that YAML output is correctly written to a file."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "yaml",
    )
    payload_in = {"factories": [{"protocol": "K", "alias": "k"}], "services": []}
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: dict(payload_in),
    )

    monkeypatch.setattr("bijux_cli.commands.dev.di.new_run_command", lambda **_: None)

    out_path = tmp_path / "di.yaml"
    dev_di_graph(
        quiet=False,
        verbose=False,
        fmt="yaml",
        pretty=True,
        debug=False,
        output=[out_path],
    )

    text = out_path.read_text("utf-8")
    loaded = yaml.safe_load(text)
    assert loaded == payload_in


def test_dev_di_graph_quiet_after_writing_exits(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that quiet mode exits after writing files without calling new_run_command."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: {"factories": [], "services": []},
    )
    out_path = tmp_path / "di.json"
    with pytest.raises(typer.Exit) as ei:
        dev_di_graph(quiet=True, output=[out_path])
    assert ei.value.exit_code == 0
    assert out_path.exists()


def test_dev_di_graph_force_serialize_failure_env(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a forced serialization failure is handled correctly."""
    monkeypatch.setenv("BIJUXCLI_TEST_FORCE_SERIALIZE_FAIL", "1")
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: {"factories": [], "services": []},
    )

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(
            msg=msg,
            code=code,
            failure=failure,
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            **kwargs,
        )
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)

    with pytest.raises(SystemExit):
        dev_di_graph(
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
            output=[],
        )

    assert called["failure"] == "serialize"
    assert called["fmt"] == "json"


@pytest.mark.parametrize("value", ["-1", "not-a-number"])
def test_dev_di_graph_invalid_limit_emits_error(
    monkeypatch: pytest.MonkeyPatch, value: str
) -> None:
    """Test that an invalid BIJUXCLI_DI_LIMIT value results in an error."""
    monkeypatch.setenv("BIJUXCLI_DI_LIMIT", value)
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: {"factories": [], "services": []},
    )

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(msg=msg, code=code, failure=failure, fmt=fmt)
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)
    with pytest.raises(SystemExit):
        dev_di_graph(
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
            output=[],
        )

    assert called["failure"] == "limit"
    assert called["fmt"] == "json"


def test_dev_di_graph_config_env_non_ascii_emits_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a non-ASCII config path in the environment results in an error."""
    monkeypatch.setenv("BIJUXCLI_CONFIG", "not-ascii-ü")
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(msg=msg, code=code, failure=failure)
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)
    with pytest.raises(SystemExit):
        dev_di_graph(
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
            output=[],
        )

    assert called["failure"] == "ascii"


def test_dev_di_graph_config_env_unreadable_path(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that an unreadable config path results in an error."""
    cfg = tmp_path / "config.yml"
    cfg.write_text("x", encoding="utf-8")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr("os.access", lambda p, mode: False)

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(msg=msg, code=code, failure=failure)
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)
    with pytest.raises(SystemExit):
        dev_di_graph(
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
            output=[],
        )

    assert called["failure"] == "config_unreadable"


def test_dev_di_graph_payload_builder_raises_value_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test that a ValueError from the payload builder is handled correctly."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: (_ for _ in ()).throw(ValueError("boom")),
    )

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(msg=msg, code=code, failure=failure)
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)
    with pytest.raises(SystemExit):
        dev_di_graph(
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
            output=[],
        )

    assert called["failure"] == "ascii"


def test_dev_di_graph_output_path_is_directory(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that providing a directory as an output file results in an error."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: {"factories": [], "services": []},
    )

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(msg=msg, code=code, failure=failure)
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)
    with pytest.raises(SystemExit):
        dev_di_graph(output=[tmp_path])

    assert called["failure"] == "output_dir"


def test_dev_di_graph_output_write_raises(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that an OSError during output file writing is handled."""
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: {"factories": [], "services": []},
    )

    out_path = tmp_path / "out.json"

    def boom(self: Path, *args: Any, **kwargs: Any) -> None:
        raise OSError("disk full")

    monkeypatch.setattr(Path, "write_text", boom, raising=True)

    called: dict[str, Any] = {}

    def _emit(
        msg: str,
        code: int,
        failure: str,
        command: str,
        fmt: str,
        quiet: bool,
        include_runtime: bool,
        debug: bool,
        **kwargs: Any,
    ) -> None:
        called.update(msg=msg, code=code, failure=failure)
        raise SystemExit(code)

    monkeypatch.setattr("bijux_cli.commands.dev.di.emit_error_and_exit", _emit)

    with pytest.raises(SystemExit):
        dev_di_graph(
            quiet=False,
            verbose=False,
            fmt="json",
            pretty=True,
            debug=False,
            output=[out_path],
        )

    assert called["failure"] == "output_write"


def test_dev_list_plugins_calls_handlers(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test that the list-plugins command correctly calls its handlers."""
    called: dict[str, Any] = {}

    def fake_validate(
        fmt: str, cmd: str, quiet: bool, include_runtime: bool | None = None
    ) -> str:
        called["validated"] = (fmt, cmd, quiet)
        return "json"

    def fake_handle(
        command: str, quiet: bool, verbose: bool, fmt: str, pretty: bool, debug: bool
    ) -> None:
        called["handled"] = (command, quiet, verbose, fmt, pretty, debug)

    monkeypatch.setattr(
        "bijux_cli.commands.dev.list_plugins.validate_common_flags", fake_validate
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.list_plugins.handle_list_plugins", fake_handle
    )

    dev_list_plugins(quiet=True, verbose=False, fmt="json", pretty=True, debug=False)

    assert called["validated"] == ("json", "dev list-plugins", True)
    assert called["handled"] == ("dev list-plugins", True, False, "json", True, False)


def test_dev_callback_returns_when_subcommand(
    ctx: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the main dev callback returns early if a subcommand is invoked."""
    ctx.invoked_subcommand = "di"
    monkeypatch.setattr(
        "bijux_cli.commands.dev.service.new_run_command",
        lambda **_: (_ for _ in ()).throw(AssertionError("should not be called")),
    )
    dev(ctx)


def test_dev_payload_basic_and_runtime_inclusion(
    ctx: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the dev payload is built correctly with and without runtime info."""
    ctx.invoked_subcommand = None

    monkeypatch.setattr(
        "bijux_cli.commands.dev.service.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )

    captured: dict[str, Any] = {}

    def fake_new_run_command(**kwargs: Any) -> None:
        captured["kwargs"] = kwargs
        built = kwargs["payload_builder"](False)
        captured["built"] = built
        if kwargs["verbose"]:
            captured["built_rt"] = kwargs["payload_builder"](True)

    monkeypatch.setattr(
        "bijux_cli.commands.dev.service.new_run_command", fake_new_run_command
    )

    dev(ctx, quiet=False, verbose=False, fmt="json", pretty=True, debug=False)
    assert captured["kwargs"]["fmt"] == "json"
    assert not captured["kwargs"]["verbose"]
    assert captured["built"]["status"] == "ok"
    assert "python" not in captured["built"]

    dev(ctx, quiet=False, verbose=True, fmt="json", pretty=False, debug=False)
    assert captured["kwargs"]["verbose"]
    assert "python" in captured["built_rt"]
    assert "platform" in captured["built_rt"]


def test_dev_payload_includes_mode_env(
    ctx: Context, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that the BIJUXCLI_DEV_MODE environment variable is included in the payload."""
    ctx.invoked_subcommand = None
    monkeypatch.setenv("BIJUXCLI_DEV_MODE", "diagnostic")
    monkeypatch.setattr(
        "bijux_cli.commands.dev.service.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )

    built_payload: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.dev.service.new_run_command",
        lambda **kw: built_payload.update(kw["payload_builder"](True)),
    )

    dev(ctx, quiet=False, verbose=True, fmt="json", pretty=True, debug=False)
    assert built_payload["status"] == "ok"
    assert built_payload["mode"] == "diagnostic"
    assert "python" in built_payload
    assert "platform" in built_payload


def test_dev_di_graph_config_env_readable_path(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a readable config file path is handled correctly."""
    cfg = tmp_path / "config.yml"
    cfg.write_text("dummy: 1", encoding="utf-8")
    monkeypatch.setenv("BIJUXCLI_CONFIG", str(cfg))

    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.validate_common_flags",
        lambda fmt, cmd, quiet, include_runtime=False: "json",
    )
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di._build_dev_di_payload",
        lambda include_runtime: {"factories": [], "services": []},
    )

    called: dict[str, Any] = {}
    monkeypatch.setattr(
        "bijux_cli.commands.dev.di.new_run_command",
        lambda **kwargs: called.update(kwargs),
    )

    dev_di_graph(
        quiet=False,
        verbose=False,
        fmt="json",
        pretty=True,
        debug=False,
        output=[],
    )

    assert called["command_name"] == "dev di"
    assert called["fmt"] == "json"
    assert called["payload_builder"](False) == {"factories": [], "services": []}
