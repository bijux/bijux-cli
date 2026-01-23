# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the Bijux CLI root api module."""

from __future__ import annotations

import asyncio
from collections.abc import Generator
from contextlib import suppress
from pathlib import Path
import sys
from types import ModuleType, SimpleNamespace
from typing import Any
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from bijux_cli.api.facade import BijuxAPI, _consume_task
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import LogLevel
from bijux_cli.core.errors import BijuxError, PluginError
from bijux_cli.plugins.contracts import RegistryProtocol
from bijux_cli.services.contracts import ObservabilityProtocol, TelemetryProtocol
from bijux_cli.services.errors import ServiceError

pytestmark = pytest.mark.filterwarnings(
    "ignore:coroutine .* was never awaited:RuntimeWarning"
)


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
        patch("bijux_cli.api.facade.Engine", return_value=mock_engine),
    ):
        mock_di.resolve.side_effect = lambda proto: {
            RegistryProtocol: mock_registry,
            ObservabilityProtocol: mock_obs,
            TelemetryProtocol: mock_tel,
        }.get(proto)
        api = BijuxAPI(log_level=LogLevel.INFO)
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

        def _run(coro: Any) -> None:
            loop = asyncio.new_event_loop()
            try:
                loop.run_until_complete(coro)
            finally:
                loop.close()

        mock_create_task = MagicMock(side_effect=_run)
        mock_loop.return_value.create_task = mock_create_task
        bijux_api._schedule_event("test", {})
    mock_create_task.assert_called_once()


def test_schedule_event_coro_no_loop(
    bijux_api: BijuxAPI, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Schedule when event returns a coroutine and no loop is running."""

    async def _evt() -> None:
        return None

    coro = _evt()
    mock_event = MagicMock(return_value=coro)
    monkeypatch.setattr(bijux_api._tel, "event", mock_event, raising=False)

    with patch("bijux_cli.api.facade.run_awaitable") as mock_run:
        mock_run.side_effect = lambda c: (c.close(), None)[1]
        bijux_api._schedule_event("test", {})

    mock_event.assert_called_once_with("test", {})
    mock_run.assert_called_once_with(coro)


def test_register(bijux_api: BijuxAPI, monkeypatch: pytest.MonkeyPatch) -> None:
    """Successful registration calls registry.register and logs once (no async warnings)."""
    monkeypatch.setattr(bijux_api, "_schedule_event", lambda *a, **k: None)
    monkeypatch.setattr(bijux_api._registry, "has", lambda _name: False, raising=False)

    captured: dict[str, Any] = {}

    def _register(name: str, obj: Any, **_kw: Any) -> None:
        captured["name"] = name
        captured["obj"] = obj

    monkeypatch.setattr(bijux_api._registry, "register", _register, raising=False)
    logs: list[tuple[str, str, dict[str, Any] | None]] = []

    def _log(level: str, msg: str, *, extra: dict[str, Any] | None = None) -> None:
        logs.append((level, msg, extra))

    monkeypatch.setattr(bijux_api._obs, "log", _log, raising=False)
    bijux_api.register("cmd", lambda: None)
    assert captured["name"] == "cmd"
    assert hasattr(captured["obj"], "execute")
    assert logs == [("info", "Registered command", {"name": "cmd"})]


def test_register_replace(bijux_api: BijuxAPI, monkeypatch: pytest.MonkeyPatch) -> None:
    """Registering an existing command should deregister then register (no AsyncMock warnings)."""
    monkeypatch.setattr(
        bijux_api, "_schedule_event", lambda *a, **k: None, raising=False
    )

    class _Tel:
        """Sync telemetry stub."""

        def event(self, *args: Any, **kwargs: Any) -> None:
            """No-op; return None."""
            return None

    class _Obs:
        """Sync observability stub."""

        def log(self, *args: Any, **kwargs: Any) -> None:
            """No-op logger."""
            return None

    monkeypatch.setattr(bijux_api, "_tel", _Tel(), raising=False)
    monkeypatch.setattr(bijux_api, "_obs", _Obs(), raising=False)
    seen: dict[str, Any] = {"has": 0, "dereg": 0, "reg": 0, "reg_args": None}

    class _Reg:
        def has(self, name: str) -> bool:
            seen["has"] += 1
            return True

        def deregister(self, name: str) -> None:
            seen["dereg"] += 1

        def register(self, name: str, obj: Any, **_kw: Any) -> None:
            seen["reg"] += 1
            seen["reg_args"] = (name, obj)

    monkeypatch.setattr(bijux_api, "_registry", _Reg(), raising=False)
    bijux_api.register("cmd", lambda: None)
    assert seen["has"] == 1
    assert seen["dereg"] == 1
    assert seen["reg"] == 1
    name, wrapper = seen["reg_args"]
    assert name == "cmd"
    assert hasattr(wrapper, "execute")


def test_register_error(bijux_api: BijuxAPI, monkeypatch: pytest.MonkeyPatch) -> None:
    """registry.register raising ServiceError is wrapped as BijuxError (no async warnings)."""
    monkeypatch.setattr(
        bijux_api, "_schedule_event", lambda *_, **__: None, raising=False
    )
    monkeypatch.setattr(bijux_api._obs, "log", lambda *_, **__: None, raising=False)

    monkeypatch.setattr(bijux_api._registry, "has", lambda _name: False, raising=False)

    def _boom(*_a: Any, **_k: Any) -> None:
        raise ServiceError("fail")

    monkeypatch.setattr(bijux_api._registry, "register", _boom, raising=False)

    with pytest.raises(BijuxError, match="Could not register"):
        bijux_api.register("cmd", lambda: None)


@pytest.mark.asyncio
async def test_run_async_success(bijux_api: BijuxAPI, mock_engine: MagicMock) -> None:
    """Tests the successful asynchronous execution of a command."""

    async def mock_run_command(*args: Any, **kwargs: Any) -> str:
        """Mocks the successful execution of a command."""
        return "result"

    mock_engine.run_command = mock_run_command
    result = await bijux_api.run_async("cmd")
    assert result == "result"


@pytest.mark.asyncio
async def test_run_async_invalid_fmt(
    bijux_api: BijuxAPI, monkeypatch: pytest.MonkeyPatch
) -> None:
    """run_async should reject unsupported formats without spawning async telemetry."""
    monkeypatch.setattr(bijux_api, "_schedule_event", lambda *a, **k: None)
    with pytest.raises(BijuxError, match="invalid is not a valid OutputFormat"):
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
    """Test that a PluginError from the engine is wrapped in a BijuxError."""
    mock_engine.run_command.side_effect = PluginError("fail")
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
    with patch("bijux_cli.api.facade.run_command") as mock_run:
        bijux_api.run_sync("cmd")
    mock_run.assert_called_once()


def test_run_sync_with_loop(bijux_api: BijuxAPI) -> None:
    """run_sync should delegate through the shared command runner."""
    with patch("bijux_cli.api.facade.run_command", return_value="ok") as mock_run:
        res = bijux_api.run_sync("anything")
    assert res == "ok"
    mock_run.assert_called_once()


def test_load_plugin(bijux_api: BijuxAPI, tmp_path: Path) -> None:
    """Test the successful loading of a plugin from a file path."""
    plugin_file = tmp_path / "plugin.py"
    plugin_file.write_text("def startup(di): pass")
    mock_plugin = MagicMock()
    mock_plugin.startup = MagicMock()
    with (
        patch("bijux_cli.plugins.load_plugin", return_value=mock_plugin),
        patch("bijux_cli.core.version", "1.0"),
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
        patch("bijux_cli.plugins.load_plugin", return_value=mock_plugin),
        patch("bijux_cli.core.version", "1.0"),
    ):
        bijux_api.load_plugin(plugin_file)
    mock_reload.assert_called_once_with(mock_module)


def test_load_plugin_error(bijux_api: BijuxAPI, tmp_path: Path) -> None:
    """Test that an error during plugin loading is wrapped in a BijuxError."""
    with (
        patch("bijux_cli.plugins.load_plugin", side_effect=Exception("fail")),
        pytest.raises(BijuxError, match="Failed to load"),
    ):
        bijux_api.load_plugin(tmp_path / "bad.py")


@pytest.mark.asyncio
async def test_wrapper_execute_sync(
    bijux_api: BijuxAPI,
    mock_registry: MagicMock,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The internal wrapper should execute a sync callback and return its result."""
    monkeypatch.setattr(
        bijux_api, "_schedule_event", lambda *_a, **_k: None, raising=False
    )
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
        patch("bijux_cli.plugins.load_plugin", return_value=mock_plugin),
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

    with patch("bijux_cli.plugins.load_plugin", return_value=mock_plugin):
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

    with patch("bijux_cli.plugins.load_plugin", return_value=mock_plugin_object):
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
        patch("bijux_cli.plugins.load_plugin", return_value=mock_plugin),
        patch("bijux_cli.core.version", "1.0"),
    ):
        bijux_api.load_plugin(plugin_file)
    mock_registry.deregister.assert_not_called()
    mock_registry.register.assert_called_once()


class _CloseAwaitable:
    """Custom awaitable with a close() hook to assert closing happens."""

    def __init__(self, ret: Any = "ok") -> None:
        """Initialize with a return value."""
        self.ret = ret
        self.closed = False

    def close(self) -> None:
        """Mark this awaitable as closed."""
        self.closed = True

    def __await__(self) -> Generator[Any, None, Any]:
        """Return an awaiter that yields the stored result."""

        async def _inner() -> Any:
            """Coroutine that returns the stored result."""
            return self.ret

        return _inner().__await__()


def test_schedule_event_non_awaitable_noop(monkeypatch: pytest.MonkeyPatch) -> None:
    """event() returns non-awaitable -> no async scheduling occurs."""
    api = BijuxAPI()

    mock_event = MagicMock(return_value=None)
    monkeypatch.setattr(api._tel, "event", mock_event, raising=False)

    api._schedule_event("name", {"a": 1})

    mock_event.assert_called_once_with("name", {"a": 1})
    # No run_awaitable call should happen when event() returns None.


def test_schedule_event_loop_no_create_task(monkeypatch: pytest.MonkeyPatch) -> None:
    """Awaitable telemetry events are routed through run_awaitable."""
    api = BijuxAPI()

    async def _ev() -> None:
        return None

    coro = _ev()
    monkeypatch.setattr(api._tel, "event", MagicMock(return_value=coro), raising=False)

    def _run_side_effect(c: Any) -> None:
        with suppress(Exception):
            c.close()

    run_spy = MagicMock(side_effect=_run_side_effect)
    with patch("bijux_cli.api.facade.run_awaitable", run_spy):
        api._schedule_event("x", {})

    run_spy.assert_called_once_with(coro)


@pytest.mark.asyncio
async def test_register_wrapper_executes_sync_and_async(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The generated wrapper .execute should run sync and async callbacks."""
    api = BijuxAPI()

    monkeypatch.setattr(api, "_schedule_event", lambda *a, **k: None)

    captured: dict[str, Any] = {}

    monkeypatch.setattr(api._registry, "has", lambda name: False, raising=False)
    monkeypatch.setattr(api._registry, "deregister", lambda name: None, raising=False)

    def _reg(name: str, obj: Any, **kw: Any) -> None:
        captured["name"] = name
        captured["obj"] = obj
        captured["kw"] = kw

    monkeypatch.setattr(api._registry, "register", _reg, raising=False)

    def _sync_cb() -> str:
        return "sync-ok"

    api.register("cmd_sync", _sync_cb)
    wrapper = captured["obj"]
    out1 = await wrapper.execute()
    assert out1 == "sync-ok"
    assert captured["name"] == "cmd_sync"

    async def _async_cb() -> str:
        return "async-ok"

    api.register("cmd_async", _async_cb)
    wrapper2 = captured["obj"]
    out2 = await wrapper2.execute()
    assert out2 == "async-ok"
    assert captured["name"] == "cmd_async"


def test_run_sync_missing_command_raises() -> None:
    """run_sync should surface command errors from the engine."""
    api = BijuxAPI()
    with pytest.raises(BijuxError, match="Failed to run command"):
        api.run_sync("anything")


@pytest.mark.asyncio
async def test_run_async_quiet_verbose_conflict(bijux_api: BijuxAPI) -> None:
    with pytest.raises(BijuxError, match="cannot be combined"):
        await bijux_api.run_async("cmd", quiet=True, verbose=True)


@pytest.mark.asyncio
async def test_run_async_schedules_success_event(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    api = BijuxAPI()

    async def _rcmd(*a: Any, **k: Any) -> str:
        return "ok"

    monkeypatch.setattr(api._engine, "run_command", _rcmd, raising=False)

    called: dict[str, Any] = {}

    def _sched(name: str, payload: dict[str, Any]) -> None:
        called["n"] = name
        called["p"] = payload

    monkeypatch.setattr(api, "_schedule_event", _sched)

    assert await api.run_async("hello") == "ok"
    assert called["n"] == "api.run"
    assert called["p"]["name"] == "hello"


@pytest.mark.asyncio
async def test_run_async_command_error_bubbled(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    api = BijuxAPI()

    async def _rcmd(*a: Any, **k: Any) -> Any:
        raise PluginError("boom")

    monkeypatch.setattr(api._engine, "run_command", _rcmd, raising=False)

    hit: dict[str, Any] = {}
    monkeypatch.setattr(api, "_schedule_event", lambda n, p: hit.setdefault("n", n))

    with pytest.raises(BijuxError, match="Failed to run command"):
        await api.run_async("x")

    assert hit.get("n") == "api.run.error"


@pytest.mark.asyncio
async def test_run_async_service_error_bubbled(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    api = BijuxAPI()

    async def _rcmd(*a: Any, **k: Any) -> Any:
        raise ServiceError("oops")

    monkeypatch.setattr(api._engine, "run_command", _rcmd, raising=False)

    seen: dict[str, Any] = {}
    monkeypatch.setattr(api, "_schedule_event", lambda n, p: seen.setdefault("n", n))

    with pytest.raises(BijuxError, match="Failed to run command"):
        await api.run_async("y")

    assert seen.get("n") == "api.run.error"


def _install_fake_module(
    monkeypatch: pytest.MonkeyPatch, name: str, **attrs: Any
) -> ModuleType:
    m = ModuleType(name)
    for k, v in attrs.items():
        setattr(m, k, v)
    monkeypatch.setitem(sys.modules, name, m)
    return m


def _make_fake_loader(monkeypatch: pytest.MonkeyPatch, plugin_obj: Any) -> ModuleType:
    mod = ModuleType("bijux_cli.plugins")

    def load_plugin(path: Path, module_name: str) -> Any:
        return plugin_obj

    mod.load_plugin = load_plugin  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "bijux_cli.plugins", mod)
    return mod


def _install_version(monkeypatch: pytest.MonkeyPatch, v: str = "1.2.3") -> None:
    _install_fake_module(monkeypatch, "bijux_cli.core.version", __version__=v)


def test_load_plugin_reload_and_register(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    api = BijuxAPI()
    _install_version(monkeypatch, "9.9.9")
    plugin = SimpleNamespace(startup=lambda di: None)
    _make_fake_loader(monkeypatch, plugin)
    p = tmp_path / "foo.py"
    p.write_text("# plugin")
    monkeypatch.setitem(sys.modules, "bijux_plugin_foo", ModuleType("bijux_plugin_foo"))

    has_mock = MagicMock(side_effect=[False, True])
    dereg_mock = MagicMock(return_value=None)
    reg_mock = MagicMock(return_value=None)

    monkeypatch.setattr(api._registry, "has", has_mock, raising=False)
    monkeypatch.setattr(api._registry, "deregister", dereg_mock, raising=False)
    monkeypatch.setattr(api._registry, "register", reg_mock, raising=False)

    with patch("importlib.reload", lambda m: m):
        api.load_plugin(p)
        dereg_mock.assert_not_called()
        reg_mock.assert_called()

        api.load_plugin(p)
        dereg_mock.assert_called_with("foo")


def test_await_maybe_non_awaitable_passthrough() -> None:
    """Tests that the _await_maybe helper passes through non-awaitable values unchanged."""
    assert BijuxAPI._await_maybe(123) == 123


@pytest.mark.asyncio
async def test_await_maybe_with_running_loop_create_task() -> None:
    """When a loop is running and has create_task, result is None and task completes."""

    async def _done() -> None:
        return None

    assert BijuxAPI._await_maybe(_done()) is None
    await asyncio.sleep(0)


def test_await_maybe_loop_run_until_complete_branch(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """If loop lacks create_task but has run_until_complete -> return its value."""
    obj = _CloseAwaitable("rv")

    def _run_until_complete(coro: Any) -> Any:
        loop = asyncio.new_event_loop()
        try:
            return loop.run_until_complete(coro)
        finally:
            loop.close()

    fake_loop = SimpleNamespace(run_until_complete=_run_until_complete)
    monkeypatch.setattr(asyncio, "get_running_loop", lambda: fake_loop)
    assert BijuxAPI._await_maybe(obj) == "rv"
    assert obj.closed is True


def test_await_maybe_ensure_future_fallback_and_close(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """No create_task & no run_until_complete -> ensure_future fails, close() still called."""
    obj = _CloseAwaitable("ignored")
    fake_loop = SimpleNamespace()
    monkeypatch.setattr(asyncio, "get_running_loop", lambda: fake_loop)

    def _ensure_future(*a: Any, **k: Any) -> Any:
        raise RuntimeError("boom")

    with patch.object(asyncio, "ensure_future", _ensure_future):
        out = BijuxAPI._await_maybe(obj, want_result=True)
    assert out is None
    assert obj.closed is True


@pytest.mark.asyncio
async def test_consume_task_swallows_task_exceptions() -> None:
    async def _bad() -> None:
        raise RuntimeError("oops")

    loop = asyncio.get_running_loop()
    t = loop.create_task(_bad())
    _consume_task(t)
    await asyncio.sleep(0)
