# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides helpers for defining plugin command groups and autocompletions.

This module offers a convenient, decorator-based API for plugin developers
to register command groups and their subcommands. It also includes a factory
function for creating dynamic shell completers for command arguments,
enhancing the interactive user experience of plugins.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

import typer

from bijux_cli.contracts import (
    ObservabilityProtocol,
    RegistryProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.di import DIContainer


def command_group(
    name: str,
    *,
    version: str | None = None,
) -> Callable[[str], Callable[[Callable[..., Any]], Callable[..., Any]]]:
    """A decorator factory for registering plugin subcommands under a group.

    This function is designed to be used as a nested decorator to easily
    define command groups within a plugin.

    Example:
        A plugin can define a "user" command group with a "create" subcommand
        like this::

            group = command_group("user", version="1.0")

            @group(sub="create")
            def create_user(username: str):
                ...

    Args:
        name (str): The name of the parent command group (e.g., "user").
        version (str | None): An optional version string for the group.

    Returns:
        Callable[[str], Callable[[Callable[..., Any]], Callable[..., Any]]]:
            A decorator that takes a subcommand name and returns the final
            decorator for the function.
    """

    def with_sub(sub: str) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        """Captures the subcommand name for registration.

        Args:
            sub (str): The name of the subcommand (e.g., "create").

        Returns:
            A decorator for the subcommand function.

        Raises:
            ValueError: If the subcommand name contains spaces.
        """
        if " " in sub:
            raise ValueError("subcommand may not contain spaces")
        full = f"{name} {sub}"

        def decorator(fn: Callable[..., Any]) -> Callable[..., Any]:
            """Registers the decorated function as a command.

            Args:
                fn (Callable[..., Any]): The function to register as a subcommand.

            Returns:
                The original, undecorated function.

            Raises:
                RuntimeError: If the `RegistryProtocol` is not initialized.
            """
            try:
                di = DIContainer.current()
                reg: RegistryProtocol = di.resolve(RegistryProtocol)
            except KeyError as exc:
                raise RuntimeError("RegistryProtocol is not initialized") from exc

            reg.register(full, fn, version=version)

            try:
                obs: ObservabilityProtocol = di.resolve(ObservabilityProtocol)
                obs.log(
                    "info",
                    "Registered command group",
                    extra={"cmd": full, "version": version},
                )
            except KeyError:
                pass

            try:
                tel: TelemetryProtocol = di.resolve(TelemetryProtocol)
                tel.event(
                    "command_group_registered", {"command": full, "version": version}
                )
            except KeyError:
                pass

            return fn

        return decorator

    return with_sub


def dynamic_choices(
    callback: Callable[[], list[str]],
    *,
    case_sensitive: bool = True,
) -> Callable[[typer.Context, typer.models.ParameterInfo, str], list[str]]:
    """Creates a `Typer` completer from a callback function.

    This factory function generates a completer that provides dynamic shell
    completion choices for a command argument or option.

    Example:
        To provide dynamic completion for a `--user` option::

            def get_all_users() -> list[str]:
                return ["alice", "bob", "carol"]

            @app.command()
            def delete(
                user: str = typer.Option(
                    ...,
                    autocompletion=dynamic_choices(get_all_users)
                )
            ):
                ...

    Args:
        callback (Callable[[], list[str]]): A no-argument function that returns
            a list of all possible choices.
        case_sensitive (bool): If True, prefix matching is case-sensitive.

    Returns:
        A `Typer` completer function.
    """

    def completer(
        ctx: typer.Context,
        param: typer.models.ParameterInfo,
        incomplete: str,
    ) -> list[str]:
        """Filters the choices provided by the callback based on user input.

        Args:
            ctx (typer.Context): The `Typer` command context.
            param (typer.models.ParameterInfo): The parameter being completed.
            incomplete (str): The current incomplete user input.

        Returns:
            list[str]: A filtered list of choices that start with the
                `incomplete` string.
        """
        items = callback()
        if case_sensitive:
            return [i for i in items if i.startswith(incomplete)]
        low = incomplete.lower()
        return [i for i in items if i.lower().startswith(low)]

    return completer


__all__ = ["command_group", "dynamic_choices"]
