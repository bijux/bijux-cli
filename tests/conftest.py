# SPDX-License-Identifier: MIT
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
from typing import Any

import pytest


@pytest.fixture(autouse=True)
def _reset_di_between_tests() -> (  # pyright: ignore[reportUnusedFunction]
    Generator[None, None, None]
):
    """Resets the `DIContainer` singleton after each test.

    This autouse fixture ensures test isolation by clearing all registered
    services and resetting the state of the dependency injection container
    after every test function completes.

    Yields:
        None: Yields control to the test function.
    """
    from bijux_cli.core.di import DIContainer

    yield
    DIContainer._reset_for_tests()  # pyright: ignore[reportPrivateUsage]


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
def _clean_env(monkeypatch: pytest.MonkeyPatch) -> None:  # pyright: ignore[reportUnusedFunction]
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
