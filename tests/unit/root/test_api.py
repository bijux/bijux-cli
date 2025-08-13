# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root api module."""

from __future__ import annotations

from pathlib import Path
import sys
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from bijux_cli.api import BijuxAPI
from bijux_cli.contracts import (
    ObservabilityProtocol,
    RegistryProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.exceptions import BijuxError, CommandError, ServiceError

# pyright: reportPrivateUsage=false


@pytest.fixture
def mock_di() -> MagicMock:
    """Provide a mock DIContainer instance."""
    return MagicMock(spec=DIContainer)


@pytest.fixture
def mock_engine() -> MagicMock:
    """Provide a mock Engine instance."""
    engine = MagicMock(spec=Engine)
    engine._di = MagicMock()
    return engine


@pytest.fixture
def mock_registry() -> MagicMock:
    """Provide a mock RegistryProtocol instance."""
    return MagicMock(spec=RegistryProtocol)


@pytest.fixture
def mock_obs() -> MagicMock:
    """Provide a mock ObservabilityProtocol instance."""
    return MagicMock(spec=ObservabilityProtocol)


@pytest.fixture
def mock_tel() -> MagicMock:
    """Provide a mock TelemetryProtocol instance."""
    return MagicMock(spec=TelemetryProtocol)


@pytest.fixture
def bijux_api(
    mock_di: MagicMock,
    mock_engine: MagicMock,
    mock_registry: MagicMock,
    mock_obs: MagicMock,
    mock_tel: MagicMock,
) -> BijuxAPI:
    """Provide a BijuxAPI instance with mocked dependencies."""
    with (
        patch.object(DIContainer, "reset"),
        patch.object(DIContainer, "current", return_value=mock_di),
        patch("bijux_cli.api.Engine", return_value=mock_engine),
    ):
        mock_di.resolve.side_effect = lambda proto: {
            RegistryProtocol: mock_registry,
            ObservabilityProtocol: mock_obs,
            TelemetryProtocol: mock_tel,
        }.get(proto)
        api = BijuxAPI(debug=False)
        return api


def test_init(bijux_api: BijuxAPI, mock_di: MagicMock, mock_engine: MagicMock) -> None:
    """Test the initialization of the BijuxAPI class."""
    assert bijux_api._di is mock_di
    assert bijux_api._engine is mock_engine
    assert bijux_api._registry
    assert bijux_api._obs
    assert bijux_api._tel


def test_schedule_event_no_coro(bijux_api: BijuxAPI) -> None:
    """Test scheduling a telemetry event that is not a coroutine."""
    bijux_api._tel.event.return_value = None  # type: ignore[attr-defined]
    bijux_api._schedule_event("test", {})
    bijux_api._tel.event.assert_called_once_with("test", {})  # type: ignore[attr-defined]


def test_schedule_event_coro_with_loop(bijux_api: BijuxAPI) -> None:
    """Test scheduling a coroutine event when an event loop is running."""
    mock_coro_func = AsyncMock()
    bijux_api._tel.event.return_value = mock_coro_func()  # type: ignore[attr-defined]
    with patch("asyncio.get_running_loop") as mock_loop:
        mock_create_task = MagicMock()
        mock_loop.return_value.create_task = mock_create_task
        bijux_api._schedule_event("test", {})
    mock_create_task.assert_called_once_with(bijux_api._tel.event.return_value)  # type: ignore[attr-defined]


def test_schedule_event_coro_no_loop(bijux_api: BijuxAPI) -> None:
    """Test scheduling a coroutine event when no event loop is running."""
    mock_coro_func = AsyncMock()
    bijux_api._tel.event.return_value = mock_coro_func()  # type: ignore[attr-defined]
    with (
        patch("asyncio.get_running_loop", side_effect=RuntimeError),
        patch("asyncio.run") as mock_run,
    ):
        bijux_api._schedule_event("test", {})
    mock_run.assert_called_once_with(bijux_api._tel.event.return_value)  # type: ignore[attr-defined]


def test_register(
    bijux_api: BijuxAPI, mock_registry: MagicMock, mock_obs: MagicMock
) -> None:
    """Test the successful registration of a new command."""
    mock_callback = MagicMock()
    mock_registry.has.return_value = False
    bijux_api.register("cmd", mock_callback)
    mock_registry.register.assert_called_once()
    mock_obs.log.assert_called_once_with(
        "info", "Registered command", extra={"name": "cmd"}
    )


def test_register_replace(bijux_api: BijuxAPI, mock_registry: MagicMock) -> None:
    """Test that registering an existing command first deregisters the old one."""
    mock_registry.has.return_value = True
    bijux_api.register("cmd", lambda: None)
    mock_registry.deregister.assert_called_once_with("cmd")


def test_register_error(bijux_api: BijuxAPI, mock_registry: MagicMock) -> None:
    """Test that an error during registration is wrapped in a BijuxError."""
    mock_registry.register.side_effect = ServiceError("fail")
    with pytest.raises(BijuxError, match="Could not register"):
        bijux_api.register("cmd", lambda: None)


@pytest.mark.asyncio
async def test_run_async_success(bijux_api: BijuxAPI, mock_engine: MagicMock) -> None:
    """Test the successful asynchronous execution of a command."""
    mock_engine.run_command = AsyncMock(return_value="result")
    result = await bijux_api.run_async("cmd")
    assert result == "result"


@pytest.mark.asyncio
async def test_run_async_invalid_fmt(bijux_api: BijuxAPI) -> None:
    """Test that run_async raises an error for an unsupported format."""
    with pytest.raises(BijuxError, match="Unsupported format"):
        await bijux_api.run_async("cmd", fmt="invalid")


@pytest.mark.asyncio
async def test_run_async_quiet_conflict(bijux_api: BijuxAPI) -> None:
    """Test that run_async raises an error for conflicting quiet/verbose flags."""
    with pytest.raises(BijuxError, match="--quiet cannot be combined"):
        await bijux_api.run_async("cmd", quiet=True, verbose=True)


@pytest.mark.asyncio
async def test_run_async_non_ascii_env(
    bijux_api: BijuxAPI, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that run_async raises an error for non-ASCII environment variables."""
    monkeypatch.setenv("TEST", "unicodé")
    with pytest.raises(BijuxError, match="Non-ASCII"):
        await bijux_api.run_async("cmd")


@pytest.mark.asyncio
async def test_run_async_command_error(
    bijux_api: BijuxAPI, mock_engine: MagicMock
) -> None:
    """Test that a CommandError from the engine is wrapped in a BijuxError."""
    mock_engine.run_command.side_effect = CommandError("fail")
    with pytest.raises(BijuxError, match="Failed to run"):
        await bijux_api.run_async("cmd")


@pytest.mark.asyncio
async def test_run_async_service_error(
    bijux_api: BijuxAPI, mock_engine: MagicMock
) -> None:
    """Test that a ServiceError from the engine is wrapped in a BijuxError."""
    mock_engine.run_command.side_effect = ServiceError("fail")
    with pytest.raises(BijuxError, match="Failed to run"):
        await bijux_api.run_async("cmd")


@pytest.mark.asyncio
async def test_run_async_generic_error(
    bijux_api: BijuxAPI, mock_engine: MagicMock
) -> None:
    """Test that a generic exception from the engine is wrapped in a BijuxError."""
    mock_engine.run_command.side_effect = ValueError("fail")
    with pytest.raises(BijuxError, match="Failed to run"):
        await bijux_api.run_async("cmd")


def test_run_sync_no_loop(bijux_api: BijuxAPI) -> None:
    """Test synchronous command execution when no event loop is running."""
    with patch("asyncio.run") as mock_run:
        bijux_api.run_sync("cmd")
    mock_run.assert_called_once()


def test_run_sync_with_loop(bijux_api: BijuxAPI) -> None:
    """Test synchronous command execution when an event loop is already running."""
    with patch("asyncio.get_running_loop") as mock_get_loop:
        mock_loop = MagicMock()
        mock_get_loop.return_value = mock_loop
        with patch.object(mock_loop, "run_until_complete") as mock_complete:
            bijux_api.run_sync("cmd")
        mock_complete.assert_called_once()


def test_load_plugin(bijux_api: BijuxAPI, tmp_path: Path) -> None:
    """Test the successful loading of a plugin from a file path."""
    plugin_file = tmp_path / "plugin.py"
    plugin_file.write_text("def startup(di): pass")
    mock_plugin = MagicMock()
    mock_plugin.startup = MagicMock()
    with (
        patch("bijux_cli.services.plugins.load_plugin", return_value=mock_plugin),
        patch("bijux_cli.__version__", "1.0"),
    ):
        bijux_api.load_plugin(plugin_file)
    mock_plugin.startup.assert_called_once_with(bijux_api._engine.di)


def test_load_plugin_reload(
    bijux_api: BijuxAPI, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that loading an already loaded plugin triggers a module reload."""
    plugin_file = tmp_path / "plugin.py"
    plugin_file.write_text("def startup(di): pass")
    module_name = f"bijux_plugin_{plugin_file.stem}"
    mock_module = MagicMock()
    monkeypatch.setitem(sys.modules, module_name, mock_module)
    mock_plugin = MagicMock()
    mock_plugin.startup = MagicMock()
    with (
        patch("importlib.reload") as mock_reload,
        patch("bijux_cli.services.plugins.load_plugin", return_value=mock_plugin),
        patch("bijux_cli.__version__", "1.0"),
    ):
        bijux_api.load_plugin(plugin_file)
    mock_reload.assert_called_once_with(mock_module)


def test_load_plugin_error(bijux_api: BijuxAPI, tmp_path: Path) -> None:
    """Test that an error during plugin loading is wrapped in a BijuxError."""
    with (
        patch("bijux_cli.services.plugins.load_plugin", side_effect=Exception("fail")),
        pytest.raises(BijuxError, match="Failed to load"),
    ):
        bijux_api.load_plugin(tmp_path / "bad.py")


@pytest.mark.asyncio
async def test_wrapper_execute_sync(
    bijux_api: BijuxAPI, mock_registry: MagicMock
) -> None:
    """Test that the internal command wrapper correctly executes a sync callback."""
    calls: list[tuple[int, int]] = []

    def cb(a: int, b: int = 0) -> int:
        calls.append((a, b))
        return a + b

    mock_registry.has.return_value = False
    bijux_api.register("add", cb)

    _, wrapper = mock_registry.register.call_args[0]
    result = await wrapper.execute(7, b=35)

    assert result == 42
    assert calls == [(7, 35)]


@pytest.mark.asyncio
async def test_wrapper_execute_async(
    bijux_api: BijuxAPI, mock_registry: MagicMock
) -> None:
    """Test that the internal command wrapper correctly executes an async callback."""

    async def acb(x: int) -> int:
        return x * 3

    mock_registry.has.return_value = False
    bijux_api.register("triple", acb)

    _, wrapper = mock_registry.register.call_args[0]
    result = await wrapper.execute(5)

    assert result == 15


def test_load_plugin_with_existing_registration(
    bijux_api: BijuxAPI, tmp_path: Path, mock_registry: MagicMock
) -> None:
    """Test that load_plugin deregisters an existing plugin before reloading."""
    plugin_file = tmp_path / "myplug.py"
    plugin_file.write_text("def startup(di): pass")
    pstem = plugin_file.stem
    mock_plugin = MagicMock()
    mock_plugin.startup = MagicMock()
    mock_registry.has.return_value = True
    with (
        patch("bijux_cli.services.plugins.load_plugin", return_value=mock_plugin),
        patch("importlib.reload"),
    ):
        bijux_api.load_plugin(plugin_file)
    mock_plugin.startup.assert_called_once_with(bijux_api._engine.di)
    mock_registry.deregister.assert_called_once_with(pstem)
    mock_registry.register.assert_called_once()


def test_load_plugin_deregisters_if_plugin_exists_v2(
    bijux_api: BijuxAPI, tmp_path: Path, mock_registry: MagicMock
) -> None:
    """Test that load_plugin deregisters a plugin if it already exists (v2)."""
    plugin_file = tmp_path / "existing_plugin.py"
    plugin_file.write_text("def startup(di): pass")
    mock_plugin = MagicMock()
    mock_plugin.startup = MagicMock()
    mock_registry.has.return_value = True

    with patch("bijux_cli.services.plugins.load_plugin", return_value=mock_plugin):
        bijux_api.load_plugin(plugin_file)

    mock_registry.has.assert_called_once_with(plugin_file.stem)
    mock_registry.deregister.assert_called_once_with(plugin_file.stem)
    mock_plugin.startup.assert_called_once()
    mock_registry.register.assert_called_once()


def test_load_plugin_deregisters_existing_plugin_v3(
    bijux_api: BijuxAPI, tmp_path: Path, mock_registry: MagicMock
) -> None:
    """Test that load_plugin deregisters a plugin if it already exists (v3)."""
    plugin_file = tmp_path / "my_plugin.py"
    plugin_file.write_text("def startup(di): pass")
    mock_registry.has.return_value = True
    mock_plugin_object = MagicMock()
    mock_plugin_object.startup = MagicMock()

    with patch(
        "bijux_cli.services.plugins.load_plugin", return_value=mock_plugin_object
    ):
        bijux_api.load_plugin(plugin_file)

    mock_registry.has.assert_called_once_with(plugin_file.stem)
    mock_registry.deregister.assert_called_once_with(plugin_file.stem)
    mock_plugin_object.startup.assert_called_once()
    mock_registry.register.assert_called_once()


def test_load_plugin_no_deregister_if_not_has(
    bijux_api: BijuxAPI, tmp_path: Path, mock_registry: MagicMock
) -> None:
    """Test that load_plugin does not deregister a plugin if it does not already exist."""
    plugin_file = tmp_path / "newplug.py"
    plugin_file.write_text("def startup(di): pass")
    mock_plugin = MagicMock()
    mock_plugin.startup = MagicMock()
    mock_registry.has.return_value = False
    with (
        patch("bijux_cli.services.plugins.load_plugin", return_value=mock_plugin),
        patch("bijux_cli.__version__", "1.0"),
    ):
        bijux_api.load_plugin(plugin_file)
    mock_registry.deregister.assert_not_called()
    mock_registry.register.assert_called_once()
