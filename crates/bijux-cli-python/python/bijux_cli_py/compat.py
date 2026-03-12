"""Compatibility aliases for older Python integration call sites."""

from __future__ import annotations

import warnings

from ._facade import command_tree_introspection, execution_facade, version


def get_version() -> str:
    warnings.warn(
        "get_version() is deprecated; use version()", DeprecationWarning, stacklevel=2
    )
    return version()


def get_command_tree() -> str:
    warnings.warn(
        "get_command_tree() is deprecated; use command_tree_introspection()",
        DeprecationWarning,
        stacklevel=2,
    )
    return command_tree_introspection()


def run_cli(argv: list[str]) -> str:
    warnings.warn(
        "run_cli() is deprecated; use execution_facade()",
        DeprecationWarning,
        stacklevel=2,
    )
    return execution_facade(argv)
