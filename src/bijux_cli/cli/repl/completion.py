# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Completion helpers for the REPL."""

from __future__ import annotations

from collections.abc import Iterator
import shlex
from typing import Any

from prompt_toolkit.completion import CompleteEvent, Completer, Completion
from prompt_toolkit.document import Document
import typer

from bijux_cli.cli.core.constants import (
    OPT_FORMAT,
    OPT_HELP,
    OPT_LOG_LEVEL,
    OPT_QUIET,
    PRETTY_FLAGS,
)

GLOBAL_OPTS = [
    *OPT_QUIET,
    *OPT_FORMAT,
    *OPT_LOG_LEVEL,
    *PRETTY_FLAGS,
    *OPT_HELP,
]

_BUILTINS = ("exit", "quit")


def _split_words(text: str) -> list[str]:
    """Split input text into shell-like words for completion."""
    try:
        words: list[str] = shlex.split(text)
    except ValueError:
        return []
    if text.endswith(" ") or not text:
        words.append("")
    return words


def _collect_completions(
    words: list[str],
    cmd_map: dict[tuple[str, ...], Any],
    builtins: tuple[str, ...],
) -> list[tuple[str, int, str | None]]:
    """Compute completion tuples for the current buffer state."""
    if not words:
        return []

    current = words[-1]
    completions: list[tuple[str, int, str | None]] = []

    if current.startswith("-"):
        completions.extend(
            (opt, -len(current), None) for opt in GLOBAL_OPTS if opt.startswith(current)
        )

    cmd_obj = None
    for i in range(len(words) - 1, 0, -1):
        key = tuple(words[:i])
        if key in cmd_map:
            cmd_obj = cmd_map[key]
            break

    if cmd_obj is None:
        completions.extend(
            (b, -len(current), None) for b in builtins if b.startswith(current)
        )

        completions.extend(
            (key[0], -len(current), None)
            for key in cmd_map
            if len(key) == 1 and key[0].startswith(current)
        )
        if words[:3] == ["config", "set", ""]:
            completions.append(("KEY=VALUE", 0, "KEY=VALUE"))
        elif not completions and current == "":
            completions.append(("DUMMY", 0, "DUMMY"))
        return completions

    is_group = hasattr(cmd_obj, "registered_commands") or hasattr(
        cmd_obj, "registered_groups"
    )
    if is_group:
        names = [c.name for c in getattr(cmd_obj, "registered_commands", [])]
        names += [g.name for g in getattr(cmd_obj, "registered_groups", [])]
        completions.extend(
            (name, -len(current), None) for name in names if name.startswith(current)
        )
    elif hasattr(cmd_obj, "params"):
        for param in cmd_obj.params:
            completions.extend(
                (opt, -len(current), None)
                for opt in (*param.opts, *(getattr(param, "secondary_opts", []) or []))
                if opt.startswith(current)
            )

    if current and "--help".startswith(current):
        completions.append(("--help", -len(current), None))

    if not completions and words[:3] == ["config", "set", ""]:
        completions.append(("KEY=VALUE", 0, "KEY=VALUE"))
    elif not completions and current == "":
        completions.append(("DUMMY", 0, "DUMMY"))
    return completions


class CommandCompleter(Completer):
    """Provides context-aware tab-completion for the REPL."""

    def __init__(self, main_app: typer.Typer) -> None:
        """Initializes the completer."""
        self.main_app = main_app
        self._cmd_map = self._collect(main_app)
        self._BUILTINS = _BUILTINS

    def _collect(
        self,
        app: typer.Typer,
        path: list[str] | None = None,
    ) -> dict[tuple[str, ...], Any]:
        """Build a command lookup table keyed by command path."""
        path = path or []
        out: dict[tuple[str, ...], Any] = {}
        for cmd in getattr(app, "registered_commands", []):
            out[tuple(path + [cmd.name])] = cmd
        for grp in getattr(app, "registered_groups", []):
            out[tuple(path + [grp.name])] = grp.typer_instance
            out.update(self._collect(grp.typer_instance, path + [grp.name]))
        return out

    def _find(
        self,
        words: list[str],
    ) -> tuple[Any | None, list[str]]:
        """Locate the deepest command match and return remaining tokens."""
        for i in range(len(words), 0, -1):
            key = tuple(words[:i])
            if key in self._cmd_map:
                return self._cmd_map[key], words[i:]
        return None, words

    def get_completions(
        self,
        document: Document,
        _complete_event: CompleteEvent,
    ) -> Iterator[Completion]:
        """Yield completions for the current prompt buffer."""
        text = document.text_before_cursor
        words = _split_words(text)
        if not words:
            return
        for value, start, display in _collect_completions(
            words, self._cmd_map, self._BUILTINS
        ):
            if display is None:
                yield Completion(value, start_position=start)
            else:
                yield Completion(value, display=display, start_position=start)
