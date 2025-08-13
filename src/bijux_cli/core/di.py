# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the central dependency injection container for the Bijux CLI.

This module defines the `DIContainer` class, a thread-safe singleton that
manages the registration, resolution, and lifecycle of all services within the
application. It allows components to be loosely coupled by requesting
dependencies based on abstract protocols rather than concrete classes.

Key features include:
    * Singleton pattern for global access via `DIContainer.current()`.
    * Thread-safe operations for concurrent environments.
    * Lazy instantiation of services upon first request.
    * Support for named registrations to allow multiple implementations of the
        same protocol.
    * Both synchronous (`resolve`) and asynchronous (`resolve_async`) service
        resolution.
    * Circular dependency detection.
"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable, Coroutine, Iterator, Sequence
from contextlib import contextmanager, suppress
from contextvars import ContextVar
import inspect
import logging
import os
from threading import RLock
from typing import Any, Literal, TypeVar, cast, overload

from injector import Binder, Injector, Module, singleton

from bijux_cli.contracts import ConfigProtocol, ObservabilityProtocol
from bijux_cli.core.exceptions import BijuxError
from bijux_cli.services.config import Config

T = TypeVar("T")
_SENTINEL = object()


def _key_name(key: type[Any] | str) -> str:
    """Returns a human-readable name for a DI service key.

    Args:
        key (type[Any] | str): The service key, which can be a type or a string.

    Returns:
        str: The string representation of the key.
    """
    if isinstance(key, str):
        return key
    try:
        return key.__name__
    except AttributeError:
        return str(key)


class AppConfigModule(Module):
    """An `injector` module for configuring core CLI dependencies."""

    def configure(self, binder: Binder) -> None:
        """Binds the `ConfigProtocol` to its default `Config` implementation.

        Args:
            binder (Binder): The `injector` binder instance.

        Returns:
            None:
        """
        binder.bind(ConfigProtocol, to=Config, scope=singleton)


class DIContainer:
    """A thread-safe, singleton dependency injection container.

    This class manages the lifecycle of services, including registration of
    factories, lazy instantiation, and resolution. It integrates with an
    underlying `injector` for basic services and handles custom named
    registrations and circular dependency detection.

    Attributes:
        _instance (DIContainer | None): The singleton instance of the container.
        _lock (RLock): A reentrant lock to ensure thread safety.
        _resolving (ContextVar): A context variable to track services currently
            being resolved, used for circular dependency detection.
        _obs (ObservabilityProtocol | None): A cached reference to the logging
            service for internal use.
        _injector (Injector): The underlying `injector` library instance.
        _store (dict): A mapping of (key, name) tuples to registered factories
            or values.
        _services (dict): A cache of resolved service instances.
    """

    _instance: DIContainer | None = None
    _lock = RLock()
    _resolving: ContextVar[set[str] | None] = ContextVar("resolving", default=None)
    _obs: ObservabilityProtocol | None = None

    @classmethod
    def current(cls) -> DIContainer:
        """Returns the active singleton instance of the `DIContainer`.

        Returns:
            DIContainer: The singleton instance.
        """
        with cls._lock:
            if cls._instance is None:
                cls._instance = cls()
                cls._log_static(
                    logging.WARNING, "DIContainer.current auto-initialized singleton"
                )
            return cls._instance

    @classmethod
    def reset(cls) -> None:
        """Resets the singleton instance, shutting down all services.

        This method is intended for use in testing environments to ensure a
        clean state between tests. It clears all registered services and
        factories.
        """
        inst = None
        with cls._lock:
            inst = cls._instance
            cls._instance = None
            cls._obs = None
        if inst is None:
            cls._log_static(logging.DEBUG, "DIContainer reset (no instance)")
            return
        try:
            asyncio.run(inst.shutdown())
        except Exception as exc:
            cls._log_static(logging.ERROR, f"Error during shutdown: {exc}")
        inst._services.clear()
        inst._store.clear()
        inst._obs = None
        cls._log_static(logging.DEBUG, "DIContainer reset")

    @classmethod
    async def reset_async(cls) -> None:
        """Asynchronously resets the singleton instance.

        This method is intended for use in testing environments. All services
        and factories are cleared.
        """
        instance = None
        with cls._lock:
            if cls._instance is not None:
                instance = cls._instance
                cls._instance = None
                cls._obs = None
        if instance is not None:
            await instance.shutdown()
            instance._services.clear()
            instance._store.clear()
            instance._obs = None
        cls._log_static(logging.DEBUG, "DIContainer reset")

    def __new__(cls) -> DIContainer:
        """Creates or returns the singleton instance of the container."""
        with cls._lock:
            if cls._instance is None:
                cls._instance = super().__new__(cls)
            return cls._instance

    def __init__(self) -> None:
        """Initializes the container's internal stores.

        This method is idempotent; it does nothing if the container has already
        been initialized.
        """
        if getattr(self, "_initialised", False):
            return
        self._injector = Injector(AppConfigModule())
        self._store: dict[
            tuple[type[Any] | str, str | None], Callable[[], Any | Awaitable[Any]] | Any
        ] = {}
        self._services: dict[tuple[type[Any] | str, str | None], Any] = {}
        self._obs: ObservabilityProtocol | None = None
        self._initialised = True
        self._log_static(logging.INFO, "DIContainer initialised")

    def register(
        self,
        key: type[T] | str,
        factory_or_value: Callable[[], T | Awaitable[T]] | T,
        name: str | None = None,
    ) -> None:
        """Registers a factory or a pre-resolved value for a given service key.

        Args:
            key (type[T] | str): The service key, which can be a protocol type
                or a unique string identifier.
            factory_or_value: The factory function that creates the service,
                or the service instance itself.
            name (str | None): An optional name for the registration, allowing
                multiple implementations of the same key.

        Returns:
            None:

        Raises:
            BijuxError: If the registration key is invalid or conflicts with an
                existing registration.
        """
        if not (isinstance(key, str) or inspect.isclass(key)):
            raise BijuxError("Service key must be a type or str", http_status=400)
        try:
            store_key = (key, name)
            if isinstance(key, str) and any(
                isinstance(k, type) and k.__name__ == key for k, _ in self._store
            ):
                raise BijuxError(
                    f"Key {key} conflicts with existing type name", http_status=400
                )
            if isinstance(key, type) and any(k == key.__name__ for k, _ in self._store):
                raise BijuxError(
                    f"Type {key.__name__} conflicts with existing string key",
                    http_status=400,
                )
            self._store[store_key] = factory_or_value
            if isinstance(factory_or_value, ObservabilityProtocol) and not isinstance(
                factory_or_value, type
            ):
                self._obs = factory_or_value
            self._log(
                logging.DEBUG,
                "Registered service",
                extra={"service_name": _key_name(key), "svc_alias": name},
            )
        except (TypeError, KeyError) as exc:
            self._log(
                logging.ERROR,
                f"Failed to register service: {exc}",
                extra={"service_name": _key_name(key), "name": name},
            )
            raise BijuxError(
                f"Failed to register service {_key_name(key)}: {exc}", http_status=400
            ) from exc

    @overload
    def _resolve_common(
        self, key: type[T] | str, name: str | None, *, async_mode: Literal[False]
    ) -> T:  # pragma: no cover
        ...

    @overload
    def _resolve_common(
        self, key: type[T] | str, name: str | None, *, async_mode: Literal[True]
    ) -> T | Awaitable[T]:  # pragma: no cover
        ...

    @overload
    def _resolve_common(
        self, key: type[T] | str, name: str | None, *, async_mode: bool
    ) -> T | Awaitable[T]:  # pragma: no cover
        ...

    def _resolve_common(
        self,
        key: type[T] | str,
        name: str | None = None,
        *,
        async_mode: bool = False,
    ) -> T | Awaitable[T]:
        """Handles the core logic for resolving a service instance.

        This internal method implements the resolution strategy:
        1. Check for a cached instance.
        2. If not cached, check for a registered factory.
        3. If no factory, attempt resolution via the underlying `injector`.
        4. If a factory is found, execute it, handling circular dependencies
           and both sync/async factories.
        5. Cache and return the result.

        Args:
            key (type[T] | str): The service key to resolve.
            name (str | None): An optional name for the registration.
            async_mode (bool): If True, allows returning an awaitable if the
                factory is async.

        Returns:
            T | Awaitable[T]: The resolved service instance, or an awaitable
                that will resolve to the instance if `async_mode` is True.

        Raises:
            KeyError: If the service is not registered.
            BijuxError: If a circular dependency is detected or the factory fails.
        """
        name_str = f"{_key_name(key)}:{name}" if name else _key_name(key)
        resolving = self._resolving.get() or set()
        if name_str in resolving:
            self._log(
                logging.ERROR,
                f"Circular dependency detected for {name_str}",
                extra={"service_name": name_str},
            )
            raise BijuxError(
                f"Circular dependency detected for {name_str}", http_status=400
            )
        with self._lock:
            store_key = (key, name)
            if (
                store_key in self._services
                and self._services[store_key] is not _SENTINEL
            ):
                self._log(
                    logging.DEBUG,
                    f"Resolved service: {type(self._services[store_key]).__name__}",
                    extra={"service_name": name_str},
                )
                return cast(T, self._services[store_key])
            if store_key not in self._store:
                if isinstance(key, type):
                    try:
                        result: T = self._injector.get(key)
                        self._services[store_key] = result
                        self._log(
                            logging.DEBUG,
                            f"Resolved service via injector: {type(result).__name__}",
                            extra={"service_name": name_str},
                        )
                        return result
                    except Exception as exc:
                        self._log(
                            logging.ERROR,
                            "Service not registered via injector",
                            extra={"service_name": name_str},
                        )
                        raise KeyError(f"Service not registered: {name_str}") from exc
                else:
                    self._log(
                        logging.ERROR,
                        "Service not registered",
                        extra={"service_name": name_str},
                    )
                    raise KeyError(f"Service not registered: {name_str}")
            self._services[store_key] = _SENTINEL
        token = self._resolving.set(resolving | {name_str})
        try:
            factory = self._store[store_key]
            is_function_like = (
                inspect.isfunction(factory)
                or inspect.ismethod(factory)
                or inspect.iscoroutinefunction(factory)
            )
            if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
                self._log(
                    logging.DEBUG,
                    f"Executing factory for service: {name_str}",
                    extra={"service_name": name_str},
                )
            raw = factory() if is_function_like else factory
            if asyncio.iscoroutine(raw):
                if async_mode:
                    return cast(Coroutine[Any, Any, T], raw)
                else:
                    coro = cast(Coroutine[Any, Any, T], raw)
                    try:
                        try:
                            loop = asyncio.get_running_loop()
                            result = loop.run_until_complete(coro)
                        except RuntimeError:
                            result = asyncio.run(coro)
                    finally:
                        with suppress(Exception):
                            if hasattr(coro, "close"):
                                coro.close()
            else:
                result = cast(T, raw)
            if result is None:
                self._log(
                    logging.ERROR,
                    "Factory returned None",
                    extra={"service_name": name_str},
                )
                raise BijuxError(
                    f"Factory for {name_str} returned None", http_status=424
                )
            with self._lock:
                self._services[store_key] = result
                if isinstance(result, ObservabilityProtocol) and not isinstance(
                    result, type
                ):
                    self._obs = result
                self._log(
                    logging.DEBUG,
                    f"Resolved service: {type(result).__name__}",
                    extra={"service_name": name_str},
                )
            return result
        except (KeyError, TypeError, RuntimeError):
            with self._lock:
                self._services.pop(store_key, None)
            self._log(
                logging.ERROR,
                f"Service resolution failed: {_key_name(key)}",
                extra={"service_name": name_str},
            )
            raise
        except BaseException as exc:
            with self._lock:
                self._services.pop(store_key, None)
            self._log(
                logging.ERROR,
                f"Factory failed: {exc}",
                extra={"service_name": name_str},
            )
            raise BijuxError(
                f"Factory for {name_str} raised: {exc}", http_status=400
            ) from exc
        finally:
            self._resolving.reset(token)

    def resolve(self, key: type[T] | str, name: str | None = None) -> T:
        """Resolves and returns a service instance synchronously.

        If the service factory is asynchronous, this method will run the
        async factory to completion.

        Args:
            key (type[T] | str): The service key to resolve.
            name (str | None): An optional name for the registration.

        Returns:
            T: The resolved service instance.

        Raises:
            KeyError: If the service is not registered.
            BijuxError: If the factory fails, returns None, or a circular
                dependency is detected.
        """
        return self._resolve_common(key, name, async_mode=False)

    async def resolve_async(self, key: type[T] | str, name: str | None = None) -> T:
        """Resolves and returns a service instance asynchronously.

        This method should be used when the caller is in an async context. It
        can resolve both synchronous and asynchronous factories.

        Args:
            key (type[T] | str): The service key to resolve.
            name (str | None): An optional name for the registration.

        Returns:
            T: The resolved service instance.

        Raises:
            KeyError: If the service is not registered.
            BijuxError: If the factory fails, returns None, or a circular
                dependency is detected.
        """
        result = self._resolve_common(key, name, async_mode=True)
        if asyncio.iscoroutine(result):
            return await cast(Awaitable[T], result)
        else:
            return cast(T, result)

    def unregister(self, key: type[Any] | str, name: str | None = None) -> bool:
        """Unregisters a service factory and removes any cached instance.

        Args:
            key (type[Any] | str): The service key to unregister.
            name (str | None): An optional name for the registration.

        Returns:
            bool: True if a service was found and unregistered, otherwise False.
        """
        with self._lock:
            store_key = (key, name)
            removed = self._store.pop(store_key, None) is not None
            if store_key in self._services and isinstance(
                self._services[store_key], ObservabilityProtocol
            ):
                self._obs = None
            self._services.pop(store_key, None)
            if removed:
                self._log(
                    logging.INFO,
                    "Unregistered service",
                    extra={"service_name": _key_name(key), "svc_alias": name},
                )
            return removed

    @contextmanager
    def override(
        self,
        key: type[T] | str,
        factory_or_value: Callable[[], T | Awaitable[T]] | T,
        name: str | None = None,
    ) -> Iterator[None]:
        """Temporarily overrides a service registration within a context block.

        This is primarily useful for testing, allowing a service to be replaced
        with a mock or stub. The original registration is restored upon exiting
        the context.

        Args:
            key (type[T] | str): The service key to override.
            factory_or_value: The temporary factory or value.
            name (str | None): An optional name for the registration.

        Yields:
            None:
        """
        with self._lock:
            store_key = (key, name)
            original_factory = self._store.get(store_key)
            original_instance = self._services.get(store_key)
            self.register(key, factory_or_value, name)
            if store_key in self._services:
                del self._services[store_key]
            self._log(
                logging.DEBUG,
                "Overriding service",
                extra={"service_name": _key_name(key), "svc_alias": name},
            )
        try:
            yield
        finally:
            with self._lock:
                if original_factory is not None:
                    self._store[store_key] = original_factory
                    if original_instance is not None:
                        self._services[store_key] = original_instance
                    else:
                        self._services.pop(store_key, None)
                    self._log(
                        logging.DEBUG,
                        "Restored service",
                        extra={"service_name": _key_name(key), "svc_alias": name},
                    )
                else:
                    self.unregister(key, name)
                    self._log(
                        logging.DEBUG,
                        "Removed service override",
                        extra={"service_name": _key_name(key), "svc_alias": name},
                    )

    async def shutdown(self) -> None:
        """Shuts down all resolved services that have a cleanup method.

        Iterates through all cached services and calls a `shutdown()` or
        `close()` method if one exists, handling both sync and async methods.
        """
        services = []
        with self._lock:
            services = list(self._services.items())
            obs_ref = self._obs
            self._services.clear()
            self._store.clear()
            self._obs = None
        for key, instance in services:
            try:
                shutdown_func = getattr(instance, "shutdown", None)
                if shutdown_func and callable(shutdown_func):
                    is_async_shutdown = asyncio.iscoroutinefunction(shutdown_func)
                    if is_async_shutdown:
                        await asyncio.wait_for(shutdown_func(), timeout=5.0)
                    else:
                        shutdown_func()
                    self._log(
                        logging.DEBUG,
                        "Shutting down service",
                        extra={"service_name": _key_name(key[0]), "svc_alias": key[1]},
                    )
                elif isinstance(instance, ObservabilityProtocol) and not isinstance(
                    instance, type
                ):
                    instance.close()
                    self._log(
                        logging.DEBUG,
                        "Closing observability service",
                        extra={"service_name": _key_name(key[0]), "svc_alias": key[1]},
                    )
            except (RuntimeError, TypeError, TimeoutError) as exc:
                self._log(
                    logging.ERROR,
                    f"Shutdown failed: {exc}",
                    extra={"service_name": _key_name(key[0]), "svc_alias": key[1]},
                )
        if obs_ref and hasattr(obs_ref, "close"):
            with suppress(Exception):
                obs_ref.close()
        self._log(logging.INFO, "DIContainer shutdown", extra={})

    def services(self) -> Sequence[tuple[type[Any] | str, str | None]]:
        """Returns a list of all resolved and cached service keys.

        Returns:
            Sequence[tuple[type[Any] | str, str | None]]: A sequence of
                (key, name) tuples for all currently resolved services.
        """
        with self._lock:
            return list(self._services.keys())

    def factories(self) -> Sequence[tuple[type[Any] | str, str | None]]:
        """Returns a list of all registered factory keys.

        Returns:
            Sequence[tuple[type[Any] | str, str | None]]: A sequence of
                (key, name) tuples for all registered factories.
        """
        with self._lock:
            return list(self._store.keys())

    def _log(
        self, level: int, msg: str, *, extra: dict[str, Any] | None = None
    ) -> None:
        """Logs a message via the resolved observability service or a fallback.

        Args:
            level (int): The logging level (e.g., `logging.INFO`).
            msg (str): The message to log.
            extra (dict[str, Any] | None): Additional context for the log entry.

        Returns:
            None:
        """
        if self._obs:
            self._obs.log(logging.getLevelName(level).lower(), msg, extra=extra or {})
        else:
            logger = logging.getLogger("bijux_cli.di")
            log_extra: dict[str, Any] = {}
            if extra:
                log_extra.update(extra)
                if "name" in log_extra:
                    log_extra["svc_alias"] = log_extra.pop("name")
            try:
                logger.log(level, msg, extra=log_extra)
            except KeyError:
                logger.warning(
                    "Failed to log with extra=%s – retrying without it", log_extra
                )
                logger.log(level, msg)

    @classmethod
    def _log_static(
        cls, level: int, msg: str, *, extra: dict[str, Any] | None = None
    ) -> None:
        """Logs a message from a class method context.

        This method attempts to use a statically cached observability service
        to prevent re-initialization loops.

        Args:
            level (int): The logging level (e.g., `logging.INFO`).
            msg (str): The message to log.
            extra (dict[str, Any] | None): Additional context for the log entry.

        Returns:
            None:
        """
        obs = cls._obs or (cls._instance._obs if cls._instance else None)
        if obs:
            obs.log(logging.getLevelName(level).lower(), msg, extra=extra or {})
        else:
            logger = logging.getLogger("bijux_cli.di")
            log_extra: dict[str, Any] = {}
            if extra:
                log_extra.update(extra)
                if "name" in log_extra:
                    log_extra["svc_alias"] = log_extra.pop("name")
            try:
                logger.log(level, msg, extra=log_extra)
            except KeyError:
                logger.log(
                    logging.WARNING,
                    "Failed to log with extra=%s – retrying without it",
                    log_extra,
                )
                logger.log(level, msg)

    @classmethod
    def _reset_for_tests(cls) -> None:
        """Fully tears down the singleton instance for testing.

        This method shuts down all services and clears all internal state of
        the singleton. It is intended exclusively for test suite cleanup.
        """
        if cls._instance:
            try:
                asyncio.run(cls._instance.shutdown())
            except Exception as exc:
                cls._log_static(logging.ERROR, f"Error during test shutdown: {exc}")
        cls._instance = None
        cls._obs = None
        cls._log_static(logging.DEBUG, "DIContainer reset for tests")


__all__ = ["DIContainer"]
