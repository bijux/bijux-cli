# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides shared utilities for the `bijux plugins` command group.

This module centralizes common logic for managing CLI plugins. It offers
helper functions for tasks such as:

* Safely traversing plugin directories for copy operations.
* Parsing metadata from `plugin.py` files without code execution by
    using the Abstract Syntax Tree (AST).
* Performing security checks, like refusing to operate on directories
    that are symbolic links.
* Validating plugin names against a standard pattern.
"""

from __future__ import annotations

from pathlib import Path
import re

PLUGIN_NAME_RE = re.compile(r"^[a-zA-Z0-9_-]+$")


def ignore_hidden_and_broken_symlinks(dirpath: str, names: list[str]) -> list[str]:
    """Creates a list of files and directories to ignore during a copy operation.

    This function is designed to be used as the `ignore` callable for
    `shutil.copytree`. It skips hidden files (starting with "."), the
    `plugin.py` file, and any broken symbolic links.

    Args:
        dirpath (str): The path to the directory being scanned.
        names (list[str]): A list of names of items within `dirpath`.

    Returns:
        list[str]: A list of item names to be ignored by `shutil.copytree`.
    """
    skip = []
    base = Path(dirpath)
    for name in names:
        if name == "plugin.py":
            continue
        if name.startswith("."):
            skip.append(name)
            continue
        entry = base / name
        if entry.is_symlink():
            try:
                entry.resolve(strict=True)
            except (FileNotFoundError, OSError):
                skip.append(name)
    return skip


def parse_required_cli_version(plugin_py: Path) -> str | None:
    """Parses `requires_cli_version` from a plugin file without executing it.

    This function safely inspects a Python file using the Abstract Syntax Tree
    (AST) to find the value of a top-level or class-level variable named
    `requires_cli_version`. This avoids the security risks of importing or
    executing untrusted code.

    Args:
        plugin_py (Path): The path to the `plugin.py` file to parse.

    Returns:
        str | None: The version specifier string if found, otherwise None.
    """
    import ast

    try:
        with plugin_py.open("r") as f:
            tree = ast.parse(f.read(), filename=str(plugin_py))
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign):
                for target in node.targets:
                    if (
                        (
                            (
                                isinstance(target, ast.Attribute)
                                and target.attr == "requires_cli_version"
                            )
                            or (
                                isinstance(target, ast.Name)
                                and target.id == "requires_cli_version"
                            )
                        )
                        and isinstance(node.value, ast.Constant)
                        and isinstance(node.value.value, str)
                    ):
                        return node.value.value
            if isinstance(node, ast.ClassDef) and node.name == "Plugin":
                for stmt in node.body:
                    if (
                        isinstance(stmt, ast.Assign)
                        and any(
                            isinstance(t, ast.Name) and t.id == "requires_cli_version"
                            for t in stmt.targets
                        )
                        and isinstance(stmt.value, ast.Constant)
                        and isinstance(stmt.value.value, str)
                    ):
                        return stmt.value.value

        return None
    except (
        FileNotFoundError,
        PermissionError,
        SyntaxError,
        UnicodeDecodeError,
        OSError,
    ):
        return None


def refuse_on_symlink(
    directory: Path,
    command: str,
    fmt: str,
    quiet: bool,
    verbose: bool,
    debug: bool,
) -> None:
    """Emits an error and exits if the given directory is a symbolic link.

    This serves as a security precaution to prevent plugin operations on
    unexpected filesystem locations.

    Args:
        directory (Path): The path to check.
        command (str): The invoking command name for the error payload.
        fmt (str): The requested output format for the error payload.
        quiet (bool): If True, suppresses output before exiting.
        verbose (bool): If True, includes runtime info in the error payload.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits the process with code 1 if `directory` is
            a symbolic link.
    """
    if directory.is_symlink():
        from bijux_cli.commands.utilities import emit_error_and_exit

        verb = command.split()[-1]
        emit_error_and_exit(
            f"Refusing to {verb}: plugins dir {directory.name!r} is a symlink.",
            code=1,
            failure="symlink_dir",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )
