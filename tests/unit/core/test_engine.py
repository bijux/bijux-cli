# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the core engine module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import asyncio
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest

from bijux_cli.contracts import ConfigProtocol, RegistryProtocol
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import CommandError
from bijux_cli.infra.observability import Observability
from bijux_cli.services.history import History


class FakeRegistry(RegistryProtocol):
    """Test double for RegistryProtocol."""

    def __init__(self) -> None:
        self.register_calls: list[tuple[str, Any, str | None]] = []
        self._store: dict[str, Any] = {}

    def register(
        self,
        name: str,
        plugin: Any,
        *,
        alias: str | None = None,
        version: str | None = None,
    ) -> None:
        """Record a registration call."""
        self.register_calls.append((name, plugin, version))
        self._store[name] = plugin

    def get(self, name: str) -> Any:
        """Get a registered plugin by name."""
        return self._store[name]

    def deregister(self, name: str) -> None:
        """Remove a registered plugin."""
        self._store.pop(name, None)

    def has(self, name: str) -> bool:
        """Return True if the name is registered."""
        return name in self._store

    def names(self) -> list[str]:
        """Return all registered names."""
        return list(self._store.keys())

    def meta(self, name: str) -> dict[str, Any]:
        """Return dummy metadata."""
        return {}

    async def call_hook(self, hook: str, *args: Any, **kwargs: Any) -> Any:
        """No-op async hook caller."""
        return None


class FakeConfig(ConfigProtocol):
    """Test double for ConfigProtocol."""

    def __init__(self, value: Any) -> None:
        self._value = value
        self._store: dict[str, Any] = {}

    def get(self, key: str, default: Any = None) -> Any:
        """Return configured timeout or default for others."""
        if key != "BIJUXCLI_COMMAND_TIMEOUT":
            return self._store.get(key, default)
        if isinstance(self._value, BaseException):
            raise self._value
        return self._value

    def all(self) -> dict[str, Any]:
        return dict(self._store)

    def set(self, key: str, value: Any) -> None:
        self._store[key] = value

    def unset(self, key: str) -> None:
        self._store.pop(key, None)

    def delete(self, key: str) -> None:
        """Alias some protocols expect; remove a key if present."""
        self._store.pop(key, None)

    def clear(self) -> None:
        self._store.clear()

    def list_keys(self) -> list[str]:
        return list(self._store.keys())

    def load(self, path: str | Path | None = None) -> None:
        """No-op for tests."""
        return None

    def save(self) -> None:
        """No-op for tests."""
        return None

    def export(self, path: str | Path, out_format: str | None = None) -> None:
        """No-op for tests."""
        return None

    def reload(self) -> None:
        """No-op for tests."""
        return None


class FakeHistory(History):
    """A fake implementation of the History service for testing."""

    def __init__(self) -> None:
        """Initialize the fake history service."""
        self.flushed = False

    def flush(self) -> None:
        """Mark the history as flushed."""
        self.flushed = True


class FakeDI:
    """A minimal DI-like container for testing the Engine."""

    def __init__(self, config: ConfigProtocol | None = None) -> None:
        """Initialize the fake DI container."""
        self._registry = FakeRegistry()
        self._history = FakeHistory()
        self._shutdown_called = False
        self._config = config if config is not None else FakeConfig(30.0)
        self._registered: dict[tuple[str, str | None], Any] = {}

    def register(
        self, key: Any, factory_or_value: Any, name: str | None = None
    ) -> None:
        """Record a registration."""
        k = key if isinstance(key, str) else getattr(key, "__name__", str(key))
        self._registered[(k, name)] = factory_or_value

    def resolve(self, key: Any) -> Any:
        """Resolve a dependency from the container."""
        if key is Observability:
            return Observability(debug=False)
        if key is RegistryProtocol:
            return self._registry
        if key is ConfigProtocol:
            return self._config
        if key is History:
            return self._history
        raise KeyError(f"Unexpected resolve: {key}")

    async def shutdown(self) -> None:
        """Mark the shutdown method as called."""
        self._shutdown_called = True


def make_plugin_dir(base: Path, name: str) -> Path:
    """Create a mock plugin directory structure."""
    folder = base / name
    src = folder / "src" / name.replace("-", "_")
    src.mkdir(parents=True)
    (src / "plugin.py").write_text("# dummy")
    return folder


@pytest.mark.parametrize(
    ("result", "expected"),
    [
        (30.0, 30.0),
        ({"value": "5"}, 5.0),
        ({"value": 7}, 7.0),
    ],
)
def test_timeout_valid_values(result: Any, expected: float) -> None:
    """Test that valid timeout configuration values are parsed correctly."""
    di = FakeDI(config=FakeConfig(result))
    eng = Engine(di=di, debug=False, fmt=OutputFormat.JSON)
    assert eng._timeout() == expected


def test_timeout_keyerror_uses_default() -> None:
    """Test that the default timeout is used when the config key is not found."""
    di = FakeDI(config=FakeConfig(KeyError("not found")))
    eng = Engine(di=di, debug=False, fmt=OutputFormat.JSON)
    assert eng._timeout() == 30.0


def test_timeout_invalid_raises_valueerror() -> None:
    """Test that an invalid timeout configuration raises a ValueError."""
    di = FakeDI(config=FakeConfig({"value": "oops"}))
    eng = Engine(di=di, debug=False, fmt=OutputFormat.JSON)
    with pytest.raises(ValueError, match="Invalid timeout configuration"):
        eng._timeout()


@pytest.mark.asyncio
async def test_run_command_success_and_exceptions() -> None:
    """Test the command execution logic for success and various failure modes."""
    di = FakeDI()
    eng = Engine(di=di, debug=False, fmt=OutputFormat.JSON)

    async def exec_ok(x: int, y: int = 1) -> int:
        return x + y

    plugin_ok = SimpleNamespace(execute=exec_ok)
    di._registry.register("add", plugin_ok, version="v1")
    result = await eng.run_command("add", 2, y=3)
    assert result == 5

    plugin_no = SimpleNamespace()
    di._registry.register("noexec", plugin_no, version="v1")
    with pytest.raises(CommandError) as e1:
        await eng.run_command("noexec")
    assert "has no callable 'execute' method" in str(e1.value)
    assert e1.value.http_status == 404

    def exec_bad(x: int) -> int:
        return x

    plugin_bad = SimpleNamespace(execute=exec_bad)
    di._registry.register("syncexec", plugin_bad, version="v1")
    with pytest.raises(CommandError) as e2:
        await eng.run_command("syncexec", 1)
    assert "is not async/coroutine" in str(e2.value)
    assert e2.value.http_status == 400


def test_shutdown_flushes_history_and_calls_di_shutdown() -> None:
    """Test that the engine shutdown process flushes history and shuts down the DI container."""
    di = FakeDI()
    eng = Engine(di=di, debug=False, fmt=OutputFormat.JSON)
    asyncio.run(eng.shutdown())
    assert di._history.flushed is True
    assert di._shutdown_called is True


def test_register_plugins_discovers_and_registers(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that plugins are correctly discovered and registered during engine initialization."""
    di = FakeDI()
    import bijux_cli.core.engine as engine_mod

    monkeypatch.setattr(engine_mod, "get_plugins_dir", lambda: tmp_path)
    calls: list[tuple[Path, str]] = []

    def fake_load_plugin(path: Path, module_name: str) -> SimpleNamespace:
        calls.append((path, module_name))

        def startup(di_arg: FakeDI) -> None:
            di_arg.register("started", True)

        return SimpleNamespace(
            name=f"nm_{module_name}",
            version="1.2.3",
            startup=startup,
        )

    monkeypatch.setattr(engine_mod, "load_plugin", fake_load_plugin)

    make_plugin_dir(tmp_path, "bijux_plugin_alpha")
    make_plugin_dir(tmp_path, "beta")
    (tmp_path / "ignore.txt").write_text("nope")

    Engine(di=di, debug=False, fmt=OutputFormat.JSON)

    assert len(calls) == 2
    assert len(di._registry.register_calls) == 2
    registered_names = {name for name, _, _ in di._registry.register_calls}
    assert registered_names == {"nm_bijux_plugin_alpha", "nm_bijux_plugin_beta"}


@pytest.mark.asyncio
async def test_run_repl_noop() -> None:
    """Test that run_repl is a no-op and returns None."""
    di = FakeDI()
    eng = Engine(di=di, debug=False, fmt=OutputFormat.JSON)
    assert await eng.run_repl() is None  # type: ignore[func-returns-value]


class DINoHistory(FakeDI):
    """A fake DI container that does not provide a History service."""

    def resolve(self, key: Any) -> Any:
        """Resolve a dependency, raising KeyError for the History service."""
        if key is History:
            raise KeyError("no history")
        return super().resolve(key)


@pytest.mark.asyncio
async def test_shutdown_without_history_calls_di_shutdown() -> None:
    """Test that shutdown succeeds even if the History service is unavailable."""
    di = DINoHistory()
    eng = Engine(di=di)
    await eng.shutdown()
    assert di._shutdown_called is True


def test_register_plugins_skips_dirs_without_plugin_py(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that directories without a plugin.py are skipped during plugin registration."""
    di = FakeDI()
    import bijux_cli.core.engine as engine_mod

    monkeypatch.setattr(engine_mod, "get_plugins_dir", lambda: tmp_path, raising=True)

    (tmp_path / "bijux_plugin_skipme").mkdir(parents=True)

    monkeypatch.setattr(
        engine_mod,
        "load_plugin",
        lambda *a, **k: (_ for _ in ()).throw(
            AssertionError("load_plugin should not be called")
        ),
        raising=True,
    )

    Engine(di=di)

    reg = di.resolve(RegistryProtocol)
    assert getattr(reg, "register_calls", []) == []


def test_register_plugins_registers_without_startup(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that a plugin is registered correctly even if it lacks a startup hook."""
    di = FakeDI()
    import bijux_cli.core.engine as engine_mod

    monkeypatch.setattr(engine_mod, "get_plugins_dir", lambda: tmp_path, raising=True)
    make_plugin_dir(tmp_path, "gamma")

    def fake_load_plugin(path: Path, module_name: str) -> SimpleNamespace:
        return SimpleNamespace(name=f"nm_{module_name}", version="0.0.1")

    monkeypatch.setattr(engine_mod, "load_plugin", fake_load_plugin, raising=True)

    Engine(di=di)

    names = [n for (n, _plugin, _ver) in di._registry.register_calls]
    assert names == ["nm_bijux_plugin_gamma"]
