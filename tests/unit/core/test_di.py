# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the core di module."""

# pyright: reportPrivateUsage=false
from __future__ import annotations

import asyncio
import builtins
from collections.abc import Awaitable
from contextlib import suppress
import logging
from typing import Any
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
from structlog.typing import FilteringBoundLogger

from bijux_cli.contracts import ConfigProtocol, ObservabilityProtocol
from bijux_cli.core.di import DIContainer, _key_name
from bijux_cli.core.exceptions import BijuxError


class DummyObs(ObservabilityProtocol):
    """A minimal mock implementation of ObservabilityProtocol for tests."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, str, dict[str, Any]]] = []
        self.closed: int = 0
        self.bound: dict[str, Any] = {}
        self.debug: bool = False
        self._logger: FilteringBoundLogger | None = None

    @classmethod
    def setup(cls, *, debug: bool = False) -> DummyObs:
        """Construct and configure the dummy observability service."""
        inst = cls()
        inst.debug = debug
        return inst

    def bind(self, **_kv: Any) -> DummyObs:
        """Record contextual key/values for subsequent logs."""
        self.bound.update(_kv)
        return self

    def log(
        self, level: str, msg: str, *, extra: dict[str, Any] | None = None
    ) -> DummyObs:
        """Record a log call and return self for chaining."""
        payload = dict(self.bound)
        if extra:
            payload.update(extra)
        self.calls.append((level, msg, payload))
        return self

    def get_logger(self) -> FilteringBoundLogger | None:
        return self._logger

    def set_telemetry(self, telemetry: Any) -> DummyObs:
        self.bound["telemetry"] = telemetry
        return self

    def close(self) -> None:
        """Record close calls."""
        self.closed += 1


class SyncService:
    """A mock service with a synchronous shutdown method."""

    def __init__(self) -> None:
        """Initialize the sync service."""
        self.shutdown_called = 0

    def shutdown(self) -> None:
        """Record a shutdown call."""
        self.shutdown_called += 1


class AsyncService:
    """A mock service with an asynchronous shutdown method."""

    def __init__(self) -> None:
        """Initialize the async service."""
        self.shutdown_called = 0

    async def shutdown(self) -> None:
        """Record an asynchronous shutdown call."""
        self.shutdown_called += 1


class BadShutdownService:
    """A mock service whose shutdown method raises an error."""

    def shutdown(self) -> None:
        """Raise an error to test shutdown error handling."""
        raise RuntimeError("boom")


def test__key_name_variants_v1() -> None:
    """Test the _key_name helper with different kinds of keys (v1)."""

    class Foo:
        pass

    assert _key_name("s") == "s"
    assert _key_name(Foo) == "Foo"


def test_current_auto_init_and_reset_paths_v1() -> None:
    """Test the singleton lifecycle of the DI container (v1)."""
    c1 = DIContainer.current()
    c2 = DIContainer.current()
    assert c1 is c2

    DIContainer.reset()
    DIContainer.reset()

    c3 = DIContainer.current()
    assert isinstance(c3, DIContainer)


@pytest.mark.asyncio
async def test_reset_async_clears_everything_v1() -> None:
    """Test that reset_async clears all registered services (v1)."""
    c = DIContainer.current()
    c.register("x", 1)
    assert c.resolve("x") == 1
    await DIContainer.reset_async()
    with pytest.raises(KeyError):
        DIContainer.current().resolve("x")


def test_register_conflict_string_then_type_v1() -> None:
    """Test that a type/string key conflict is detected (v1)."""

    class Foo:
        pass

    c = DIContainer.current()
    c.register("Foo", 1)
    with pytest.raises(BijuxError, match="Type Foo conflicts with existing string key"):
        c.register(Foo, Foo())


def test_register_conflict_type_then_string_v1() -> None:
    """Test that a string/type key conflict is detected (v1)."""

    class Bar:
        pass

    c = DIContainer.current()
    c.register(Bar, Bar())
    with pytest.raises(BijuxError, match="Key Bar conflicts with existing type name"):
        c.register("Bar", 1)


def test_register_and_resolve_value_and_missing_v1() -> None:
    """Test basic registration, resolution, and handling of missing keys (v1)."""
    c = DIContainer.current()
    c.register("Answer", 42)
    assert c.resolve("Answer") == 42
    assert ("Answer", None) in c.factories()
    assert ("Answer", None) in c.services()
    with pytest.raises(KeyError):
        c.resolve("Unknown")


def test_logging_via_observability_and_keyerror_fallback_v1(
    monkeypatch: pytest.MonkeyPatch, caplog: pytest.LogCaptureFixture
) -> None:
    """Test logging through the observability service and its fallback (v1)."""
    c = DIContainer.current()
    obs = DummyObs()
    c.register("obs", obs)
    c._log(logging.INFO, "hello", extra={"name": "svc"})
    assert obs.calls
    assert obs.calls[-1][0] == "info"

    DIContainer._log_static(logging.WARNING, "world", extra={"name": "svc2"})
    assert any(m for m in obs.calls if m[0] == "warning")

    c._obs = None
    with caplog.at_level(logging.WARNING, logger="bijux_cli.di"):
        c._log(logging.INFO, "fallback-instance", extra={"message": "oops"})
        DIContainer._obs = None
        DIContainer._log_static(
            logging.INFO, "fallback-static", extra={"message": "oops2"}
        )


def test_resolve_coroutine_sync_via_asyncio_run_v1(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test synchronous resolution of a coroutine factory via asyncio.run (v1)."""
    c = DIContainer.current()

    async def maker() -> str:
        return "ok-sync"

    monkeypatch.setenv("VERBOSE_DI", "1")
    monkeypatch.delenv("BIJUXCLI_TEST_MODE", raising=False)

    c.register("AsyncSvc", maker)  # type: ignore[arg-type]
    assert c.resolve("AsyncSvc") == "ok-sync"


def test_resolve_coroutine_sync_with_fake_running_loop_v1(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test synchronous resolution of a coroutine factory with a fake running loop (v1)."""
    c = DIContainer.current()

    async def maker() -> str:
        return "ok-loop"

    class FakeLoop:
        def run_until_complete(self, _coro: Any) -> str:
            return "ok-loop"

    monkeypatch.setattr(asyncio, "get_running_loop", FakeLoop)
    c.register("AsyncSvc2", maker)  # type: ignore[arg-type]
    assert c.resolve("AsyncSvc2") == "ok-loop"


@pytest.mark.asyncio
async def test_resolve_async_returns_awaited_value_v1() -> None:
    """Test that resolve_async correctly awaits and returns the factory result (v1)."""
    c = DIContainer.current()

    async def maker() -> int:
        return 123

    c.register("AsyncSvc3", maker)  # type: ignore[arg-type]
    assert await c.resolve_async("AsyncSvc3") == 123


def test_factory_returns_none_raises_v1() -> None:
    """Test that a factory returning None raises a BijuxError (v1)."""

    c = DIContainer.current()

    async def _none_factory() -> None:
        """Async factory that returns None."""
        return None

    c.register("NoneFactory", _none_factory)  # type: ignore[arg-type]

    with pytest.raises(BijuxError, match="returned None"):
        c.resolve("NoneFactory")


def test_factory_raises_typeerror_is_reraised_v1() -> None:
    """Test that a TypeError from a factory is re-raised directly (v1)."""
    c = DIContainer.current()

    def bad() -> None:
        """Always raises a TypeError."""
        raise TypeError("bad")

    c.register("BadType", bad)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        c.resolve("BadType")


def test_factory_raises_other_exception_is_wrapped_v1() -> None:
    """Test that other exceptions from a factory are wrapped in BijuxError (v1)."""
    c = DIContainer.current()

    def bad() -> None:
        """Always raises a ValueError."""
        raise ValueError("boom")

    c.register("BadOther", bad)  # type: ignore[arg-type]
    with pytest.raises(BijuxError, match="Factory for BadOther raised: boom"):
        c.resolve("BadOther")


def test_circular_dependency_detection_v1() -> None:
    """Test that a circular dependency during resolution is detected (v1)."""
    c = DIContainer.current()

    def make_a() -> Any:
        """Resolve and return service 'A' from the container."""
        return c.resolve("A")

    c.register("A", make_a)
    with pytest.raises(BijuxError, match="Circular dependency detected for A"):
        c.resolve("A")


def test_override_restores_original_v1() -> None:
    """Test that the override context manager correctly restores an original value (v1)."""
    c = DIContainer.current()
    c.register("X", 1)
    assert c.resolve("X") == 1
    with c.override("X", 2):
        assert c.resolve("X") == 2
    assert c.resolve("X") == 1


def test_override_then_remove_for_missing_registration_v1() -> None:
    """Test that overriding a non-existent key is correctly removed on exit (v1)."""
    c = DIContainer.current()
    with c.override("Y", 9):
        assert c.resolve("Y") == 9
    with pytest.raises(KeyError):
        c.resolve("Y")


def test_unregister_removes_and_clears_obs_when_service_is_obs_v1() -> None:
    """Test that unregistering the observability service also clears the internal cache (v1)."""
    c = DIContainer.current()
    obs = DummyObs()
    c.register("obs", obs)
    assert c.resolve("obs") is obs
    ok = c.unregister("obs")
    assert ok is True
    with pytest.raises(KeyError):
        c.resolve("obs")


def test_services_and_factories_views_v1() -> None:
    """Test the factories() and services() view methods (v1)."""
    c = DIContainer.current()
    c.register("a", 1)

    async def factory_b() -> int:
        return 2

    c.register("b", factory_b)  # type: ignore[arg-type]
    facts = set(c.factories())
    assert ("a", None) in facts
    assert ("b", None) in facts

    _ = c.resolve("a")
    _ = c.resolve("b")

    srvs = set(c.services())
    assert ("a", None) in srvs
    assert ("b", None) in srvs


@pytest.mark.asyncio
async def test_shutdown_handles_sync_async_obs_and_error_paths_v1() -> None:
    """Test the shutdown process with various service types (v1)."""
    c = DIContainer.current()
    sync = SyncService()
    async_svc = AsyncService()
    bad = BadShutdownService()
    obs = DummyObs()

    c.register("sync", sync)
    c.register("async", async_svc)
    c.register("bad", bad)
    c.register("obs", obs)

    assert c.resolve("sync") is sync
    assert c.resolve("async") is async_svc
    assert c.resolve("bad") is bad
    assert c.resolve("obs") is obs

    await c.shutdown()

    assert sync.shutdown_called == 1
    assert async_svc.shutdown_called == 1
    assert obs.closed >= 1
    assert not c.factories()
    assert not c.services()


def test_resolve_via_injector_when_not_registered_plain_type_v1() -> None:
    """Test resolving an unregistered concrete type via the injector (v1)."""
    c = DIContainer.current()

    class Plain:
        pass

    obj = c.resolve(Plain)
    assert isinstance(obj, Plain)


def test_resolve_via_injector_missing_binding_raises_key_error_v1() -> None:
    """Test that resolving an unmapped protocol raises a KeyError (v1)."""
    c = DIContainer.current()

    with pytest.raises(KeyError):
        c.resolve(ConfigProtocol)


def test_log_and_log_static_key_error_fallback_v1(
    caplog: pytest.LogCaptureFixture,
) -> None:
    """Test the logger's fallback mechanism for KeyError (v1)."""
    c = DIContainer.current()
    c._obs = None
    with caplog.at_level(logging.WARNING, logger="bijux_cli.di"):
        c._log(logging.INFO, "msg1", extra={"message": "oops"})
        DIContainer._obs = None
        DIContainer._log_static(logging.INFO, "msg2", extra={"message": "oops2"})


def test_init_idempotent_v1() -> None:
    """Test that initializing the container is idempotent (v1)."""
    a = DIContainer.current()
    b = DIContainer()
    assert a is b


def test_key_name_variants() -> None:
    """Test the _key_name helper with different kinds of keys."""

    class MyClass:
        pass

    assert _key_name("my-key") == "my-key"
    assert _key_name(MyClass) == "MyClass"


def test_singleton_and_idempotent_init() -> None:
    """Verify that .current() and __new__ return the same instance."""
    c1 = DIContainer.current()
    c2 = DIContainer.current()
    c3 = DIContainer()
    assert c1 is c2 is c3
    c1._injector = MagicMock()
    DIContainer()
    assert isinstance(c1._injector, MagicMock)


def test_reset_with_instance_and_error_on_shutdown(
    caplog: pytest.LogCaptureFixture, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test the reset method's error handling path during shutdown."""
    c = DIContainer.current()
    mock_shutdown = AsyncMock(side_effect=RuntimeError("Shutdown boom"))
    monkeypatch.setattr(c, "shutdown", mock_shutdown)

    with caplog.at_level(logging.ERROR, logger="bijux_cli.di"):
        DIContainer.reset()

    assert "Error during shutdown: Shutdown boom" in caplog.text
    assert DIContainer._instance is None


def test_reset_with_no_instance(caplog: pytest.LogCaptureFixture) -> None:
    """Test the reset method's path when no instance exists."""
    with caplog.at_level(logging.DEBUG, logger="bijux_cli.di"):
        DIContainer.reset()
    assert "DIContainer reset (no instance)" in caplog.text


@pytest.mark.asyncio
async def test_reset_async_with_and_without_instance() -> None:
    """Test both branches of the reset_async method."""
    await DIContainer.reset_async()
    c = DIContainer.current()
    c.register("test", 1)
    await DIContainer.reset_async()
    assert DIContainer._instance is None


def test_register_invalid_key_type_v1() -> None:
    """Test that registering a service with an invalid key type raises an error."""
    c = DIContainer.current()
    with pytest.raises(BijuxError, match="Service key must be a type or str"):
        c.register(
            123,  # type: ignore[arg-type]
            "value",
        )


def test_register_type_string_conflicts() -> None:
    """Test that registering a type and a string with the same name raises an error."""

    class MyService:
        pass

    c = DIContainer.current()
    c.register(MyService, MyService())
    with pytest.raises(
        BijuxError, match="Key MyService conflicts with existing type name"
    ):
        c.register("MyService", "value2")
    c.unregister(MyService)
    c.register("MyService", "value2")
    with pytest.raises(
        BijuxError, match="Type MyService conflicts with existing string key"
    ):
        c.register(MyService, MyService())


def test_register_internal_type_error_is_wrapped(
    caplog: pytest.LogCaptureFixture,
) -> None:
    """Test that an internal TypeError during registration is wrapped in a BijuxError."""
    c = DIContainer.current()
    with (
        patch.object(
            c,
            "_store",
            MagicMock(__iter__=MagicMock(side_effect=TypeError("mocked iter error"))),
        ),
        pytest.raises(BijuxError, match="Failed to register service"),
    ):
        c.register("key", "value")
    assert "Failed to register service: mocked iter error" in caplog.text


def test_resolve_circular_dependency() -> None:
    """Test that a circular dependency during resolution is detected."""
    c = DIContainer.current()
    c.register("A", lambda: c.resolve("A"))
    with pytest.raises(BijuxError, match="Circular dependency detected for A"):
        c.resolve("A")


@pytest.mark.asyncio
async def test_resolve_async_with_sync_factory() -> None:
    """Test that resolve_async can correctly resolve a synchronous factory."""
    c = DIContainer.current()
    c.register("sync-val", lambda: 42)  # type: ignore[arg-type, return-value]
    assert await c.resolve_async("sync-val") == 42


class ClosableCoroService:
    """A mock service whose factory returns a coroutine with a close method."""

    @staticmethod
    async def factory() -> Awaitable[Any]:
        """Create a mock coroutine with a close method."""
        coro = asyncio.sleep(0)
        coro.close = MagicMock()  # type: ignore[method-assign]
        return coro


def test_factory_returns_none_raises_error() -> None:
    """Test that a factory returning None raises a BijuxError."""
    c = DIContainer.current()
    c.register("NoneFactory", lambda: None)  # type: ignore[arg-type, return-value]
    with pytest.raises(BijuxError, match="Factory for NoneFactory returned None"):
        c.resolve("NoneFactory")


def test_factory_raises_base_exception_is_wrapped() -> None:
    """Test that a BaseException from a factory is wrapped in a BijuxError."""
    c = DIContainer.current()
    c.register("SystemExitFactory", lambda: (_ for _ in ()).throw(SystemExit))
    with pytest.raises(BijuxError, match="Factory for SystemExitFactory raised"):
        c.resolve("SystemExitFactory")


def test_unregister_nonexistent_key(caplog: pytest.LogCaptureFixture) -> None:
    """Test that unregistering a non-existent key returns False and does not log."""
    c = DIContainer.current()
    with caplog.at_level(logging.INFO):
        assert c.unregister("nonexistent") is False
    assert "Unregistered service" not in caplog.text


def test_override_restores_unresolved_service() -> None:
    """Test that the override context manager correctly restores an unresolved service."""
    c = DIContainer.current()
    c.register("service", lambda: "original")  # type: ignore[arg-type, return-value]
    with c.override("service", lambda: "overridden"):  # type: ignore[arg-type, return-value]
        assert c.resolve("service") == "overridden"
    assert c.resolve("service") == "original"


def test_override_restores_original_resolved_instance() -> None:
    """Test that the override context manager correctly restores a resolved instance."""
    c = DIContainer.current()
    c.register("service", "original_instance")
    assert c.resolve("service") == "original_instance"
    with c.override("service", "overridden_instance"):
        assert c.resolve("service") == "overridden_instance"
    assert c.resolve("service") == "original_instance"


def test_reset_for_tests_logs_shutdown_error(
    caplog: pytest.LogCaptureFixture, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Test that an error during test reset shutdown is logged correctly."""
    c = DIContainer.current()
    monkeypatch.setattr(c, "shutdown", AsyncMock(side_effect=RuntimeError("bad-reset")))

    with caplog.at_level(logging.ERROR, logger="bijux_cli.di"):
        DIContainer._reset_for_tests()

    assert "Error during test shutdown: bad-reset" in caplog.text
    assert DIContainer._instance is None


def test_instance_log_key_error_fallback(caplog: pytest.LogCaptureFixture) -> None:
    """Test the DI instance logger's fallback mechanism for KeyError."""
    c = DIContainer.current()

    orig_log = logging.Logger.log

    def log_maybe_keyerror(
        self: logging.Logger, level: int, msg: str, *args: Any, **kwargs: Any
    ) -> None:
        """A mock logger method that raises a `KeyError` if `extra` is passed in kwargs."""
        if "extra" in kwargs and kwargs["extra"] is not None:
            raise KeyError("Injected for fallback")
        return orig_log(self, level, msg, *args, **kwargs)

    with (
        patch.object(logging.Logger, "log", log_maybe_keyerror),
        caplog.at_level(logging.WARNING, logger="bijux_cli.di"),
    ):
        c._log(logging.INFO, "instance-fallback", extra={"anything": "x"})

    assert "Failed to log with extra=" in caplog.text


def test_resolve_coroutine_hasattr_false_branch(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test resolving a coroutine that lacks a 'close' method."""
    c = DIContainer.current()

    async def maker() -> int:
        """A simple coroutine function that returns a fixed integer value."""
        return 123

    class FakeLoop:
        """A fake asyncio event loop for synchronous unit testing of async calls."""

        @staticmethod
        def run_until_complete(coro: Any) -> str:
            """Simulates running a coroutine, attempts cleanup, and returns a fixed string."""
            with suppress(Exception):
                coro.close()
            return "ok-no-close"

    monkeypatch.setattr(asyncio, "get_running_loop", FakeLoop)

    c.register("AsyncSvcNoClose", maker)  # type: ignore[arg-type]

    real_hasattr = builtins.hasattr

    def fake_hasattr(obj: Any, name: str) -> bool:
        """A mock of `hasattr` that always returns `False` for the 'close' attribute."""
        if name == "close":
            return False
        return real_hasattr(obj, name)

    monkeypatch.setattr(builtins, "hasattr", fake_hasattr)

    assert c.resolve("AsyncSvcNoClose") == "ok-no-close"


def test_static_log_key_error_fallback_and_name_mapping(
    caplog: pytest.LogCaptureFixture,
) -> None:
    """Test the static logger's KeyError fallback and name mapping."""
    orig_log = logging.Logger.log

    def log_maybe_keyerror(
        self: logging.Logger, level: int, msg: str, *args: Any, **kwargs: Any
    ) -> None:
        if "extra" in kwargs and kwargs["extra"] is not None:
            raise KeyError("Injected for static fallback")
        return orig_log(self, level, msg, *args, **kwargs)

    with (
        patch.object(logging.Logger, "log", log_maybe_keyerror),
        caplog.at_level(logging.WARNING, logger="bijux_cli.di"),
    ):
        DIContainer._log_static(
            logging.INFO,
            "static-fallback",
            extra={"name": "svc", "message": "oops"},
        )

    assert "Failed to log with extra=" in caplog.text
