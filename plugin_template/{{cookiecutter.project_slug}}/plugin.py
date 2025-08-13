# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""An example plugin demonstrating integration with a CLI application.

This module provides a simple command-line interface using Typer,
a basic health check function, and a class representing the core
plugin logic. It is intended to serve as a template or reference for
creating new plugins.
"""

from __future__ import annotations

from typing import Any

import typer

app = typer.Typer(
    help="An example plugin providing a sample 'run' command.",
)


@app.command()
def run(
    input_string: str = typer.Argument(
        ..., help="The input string to be processed and printed."
    ),
) -> None:
    """Processes an input string and prints a greeting.

    This command serves as the primary entry point for the plugin's
    functionality, demonstrating how to accept and handle user input.
    It prints the provided string to standard output, prefixed with a
    greeting message.

    Args:
        input_string: The string value provided by the user to be
                      included in the output message.
    """
    print(f"Hello from plugin, got: {input_string}")


def health(di: Any | None = None) -> bool:
    """Performs a health check for the plugin.

    This function is designed to be called by the main application to
    verify the operational status of the plugin. In this example, it
    always returns True, indicating a healthy state.

    The 'di' parameter is included to demonstrate compatibility with
    dependency injection systems, though it is not used in this
    basic implementation.

    Args:
        di: An optional dependency injection container or context.
            Defaults to None.

    Returns:
        True if the plugin is healthy, False otherwise.
    """
    return True


class Plugin:
    """Encapsulates the core logic and functionality of the plugin.

    This class serves as a container for the plugin's business logic,
    separating it from the command-line interface definitions. It allows
    the core functionality to be instantiated and used independently.
    """

    def run(self, input_value: Any) -> Any:
        """A method representing the main execution logic of the plugin.

        In this example, the method acts as an identity function, simply
        returning the value that was passed to it. In a real-world
        plugin, this method would contain the primary processing logic.

        Args:
            input_value: The data to be processed by the plugin.

        Returns:
            The processed result, which in this case is the original
            input value.
        """
        return input_value