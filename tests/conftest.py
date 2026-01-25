# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Provides shared `pytest` fixtures for the Bijux CLI test suite.

This module defines a set of `pytest` fixtures that are automatically
applied to all tests. These fixtures are crucial for ensuring test isolation
and providing a consistent, clean environment for each test case.

The provided fixtures handle:
* Resetting the singleton `DIContainer` between tests to prevent state leakage.
* Cleaning up specific `BIJUXCLI_*` environment variables to avoid
    interference from the host shell.
* Providing a convenient helper for applying multiple monkeypatches.
"""

from __future__ import annotations

from collections.abc import Generator
import contextlib
import os
from pathlib import Path
import tempfile
from typing import Any

import pytest

_DEFAULT_PLUGINS_DIR = Path(tempfile.mkdtemp(prefix="bijuxcli-plugins-")).resolve()
os.environ.setdefault("BIJUXCLI_PLUGINS_DIR", str(_DEFAULT_PLUGINS_DIR))


@pytest.fixture(autouse=True)
def _reset_di_between_tests() -> Generator[None, None, None]:
    """Resets the `DIContainer` singleton after each test.

    This autouse fixture ensures test isolation by clearing all registered
    services and resetting the state of the dependency injection container
    after every test function completes.

    Yields:
        None: Yields control to the test function.
    """
    from bijux_cli.core.di import DIContainer

    yield
    DIContainer._reset_for_tests()


@pytest.fixture(autouse=True)
def _register_basic_serializer() -> None:
    """Ensure a minimal serializer is registered for tests needing DI output."""
    import json as _json

    from bijux_cli.core.di import DIContainer
    from bijux_cli.core.enums import OutputFormat
    from bijux_cli.infra.contracts import Serializer

    class _BasicSerializer:
        def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
            if fmt not in (OutputFormat.JSON, OutputFormat.YAML):
                raise ValueError("Unsupported format in tests")
            payload = _normalize_payload(obj)
            return _json.dumps(payload, indent=2 if pretty else None)

        def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
            return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

        def loads(self, data: str | bytes, *, fmt: OutputFormat, pretty: bool) -> Any:
            _ = (fmt, pretty)
            return _json.loads(data)

        def emit(self, payload: Any, *, fmt: OutputFormat, pretty: bool) -> None:
            _ = (payload, fmt, pretty)
            return None

    def _normalize_payload(obj: Any) -> Any:
        if hasattr(obj, "__dataclass_fields__"):
            data = {}
            for key, value in obj.__dict__.items():
                if value is None:
                    continue
                data[key] = _normalize_payload(value)
            return data
        if isinstance(obj, dict):
            return {key: _normalize_payload(value) for key, value in obj.items()}
        if isinstance(obj, list | tuple | set):
            return [_normalize_payload(value) for value in obj]
        if hasattr(obj, "value"):
            return obj.value
        return obj

    di = DIContainer.current()
    di.register(Serializer, lambda: _BasicSerializer())


@pytest.fixture(autouse=True)
def helpers(monkeypatch: pytest.MonkeyPatch) -> Any:
    """Provides a helper class for managing multiple monkeypatches.

    This fixture attaches a `Helpers` class to the `pytest` namespace, which
    contains a context manager for applying multiple patches at once.

    Args:
        monkeypatch: The `pytest` `monkeypatch` fixture.

    Returns:
        The `Helpers` class, making it available to tests.
    """

    class Helpers:
        """A container for test helper methods."""

        @staticmethod
        @contextlib.contextmanager
        def apply(*patches: Any) -> Generator[None, None, None]:
            """Applies one or more monkeypatches within a context manager block.

            Args:
                *patches: A variable number of monkeypatch objects to apply.

            Yields:
                None: Yields control to the `with` block where the patches
                    are active.
            """
            for p in patches:
                p.start()
            try:
                yield
            finally:
                for p in reversed(patches):
                    p.stop()

    pytest.helpers = Helpers  # type: ignore[attr-defined]
    return Helpers


@pytest.fixture(autouse=True)
def _clean_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """Removes potentially interfering environment variables before each test.

    This auto-use fixture ensures test isolation by unsetting specific
    `BIJUXCLI_*` environment variables that might be present in the host shell,
    preventing them from affecting test outcomes.

    Args:
        monkeypatch: The `pytest` `monkeypatch` fixture.

    Returns:
        None:
    """
    vars_to_remove = [
        "BIJUXCLI_HISTORY_FILE",
        "BIJUXCLI_CONFIG",
        "BIJUXCLI_PLUGINS_DIR",
        "BIJUXCLI_DOCS_DIR",
    ]
    for var in vars_to_remove:
        monkeypatch.delenv(var, raising=False)


@pytest.fixture(autouse=True)
def _isolate_plugins_dir(monkeypatch: pytest.MonkeyPatch, tmp_path: Any) -> None:
    """Force tests to use an isolated plugins directory."""
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path / "plugins"))


def pytest_collection_modifyitems(
    config: pytest.Config, items: list[pytest.Item]
) -> None:
    """Apply unit/integration/night markers based on test location."""
    for item in items:
        path_str = str(item.fspath)
        if "/tests/unit/" in path_str:
            item.add_marker("unit")
        elif "/tests/regression/" in path_str:
            item.add_marker("integration")
        elif "/tests/e2e/" in path_str:
            item.add_marker("e2e")
            item.add_marker("integration")
        elif "/tests/night/" in path_str:
            item.add_marker("night")
            item.add_marker("slow")
        timeout_marker = item.get_closest_marker("timeout")
        if timeout_marker and timeout_marker.args:
            try:
                seconds = float(timeout_marker.args[0])
            except (TypeError, ValueError):
                seconds = None
            if seconds is not None and seconds > 10:
                item.add_marker("slow")
