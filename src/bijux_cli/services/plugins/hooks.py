# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the hook specifications for the Bijux CLI plugin system.

This module uses `pluggy`'s hook specification markers to define the formal
set of hooks that plugins can implement. The `CoreSpec` class groups all
standard lifecycle and execution hooks that the core application will invoke
on registered plugins. This provides a clear and versioned interface for
plugin developers.
"""

from __future__ import annotations

from typing import Any

import pluggy

import bijux_cli
from bijux_cli.contracts import ObservabilityProtocol
from bijux_cli.core.di import DIContainer

PRE_EXECUTE = "pre_execute"
POST_EXECUTE = "post_execute"
SPEC_VERSION = bijux_cli.version

hookspec = pluggy.HookspecMarker("bijux")


class CoreSpec:
    """Defines the core hook specifications for CLI plugins.

    Plugins can implement methods matching these specifications to integrate with
    the CLI's lifecycle and command execution flow.

    Attributes:
        _log (ObservabilityProtocol): The logging service.
    """

    def __init__(self, dependency_injector: DIContainer) -> None:
        """Initializes the `CoreSpec`.

        Args:
            dependency_injector (DIContainer): The DI container for resolving
                services like the logger.
        """
        self._log = dependency_injector.resolve(ObservabilityProtocol)

    @hookspec
    async def startup(self) -> None:
        """A hook that is called once when the CLI engine starts up.

        Plugins can use this hook to perform initialization tasks, such as
        setting up resources or starting background tasks.
        """
        self._log.log("debug", "Hook startup called", extra={})

    @hookspec
    async def shutdown(self) -> None:
        """A hook that is called once when the CLI engine shuts down.

        Plugins can use this hook to perform cleanup tasks, such as releasing
        resources or flushing buffers.
        """
        self._log.log("debug", "Hook shutdown called", extra={})

    @hookspec
    async def pre_execute(
        self, name: str, args: tuple[Any, ...], kwargs: dict[str, Any]
    ) -> None:
        """A hook that is called immediately before a command is executed.

        Args:
            name (str): The name of the command to be executed.
            args (tuple[Any, ...]): The positional arguments passed to the command.
            kwargs (dict[str, Any]): The keyword arguments passed to the command.
        """
        self._log.log(
            "debug",
            "Hook pre_execute called",
            extra={"name": name, "args": args, "kwargs": kwargs},
        )

    @hookspec
    async def post_execute(self, name: str, result: Any) -> None:
        """A hook that is called immediately after a command has executed.

        Args:
            name (str): The name of the command that was executed.
            result (Any): The result object returned from the command.
        """
        self._log.log(
            "debug",
            "Hook post_execute called",
            extra={"name": name, "result": repr(result)},
        )

    @hookspec
    def health(self) -> bool | str:
        """A hook to check the health of a plugin.

        This hook is called by the `bijux plugins check` command. A plugin can
        return a boolean or a string to indicate its status.

        Returns:
            bool | str: `True` for healthy, `False` for unhealthy, or a string
                with a descriptive status message.
        """
        self._log.log("debug", "Hook health called", extra={})
        return True


__all__ = ["CoreSpec", "SPEC_VERSION", "PRE_EXECUTE", "POST_EXECUTE"]
