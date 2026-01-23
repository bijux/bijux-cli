# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Completion helpers for the REPL."""

from __future__ import annotations

from collections.abc import Iterator
import shlex
from typing import Any

from prompt_toolkit.completion import CompleteEvent, Completer, Completion
from prompt_toolkit.document import Document
import typer

from bijux_cli.cli.constants import (
    OPT_FORMAT,
    OPT_HELP,
    OPT_LOG_LEVEL,
    OPT_QUIET,
    OPT_VERBOSE,
    PRETTY_FLAGS,
)

GLOBAL_OPTS = [
    *OPT_QUIET,
    *OPT_VERBOSE,
    *OPT_FORMAT,
    *OPT_LOG_LEVEL,
    *PRETTY_FLAGS,
    *OPT_HELP,
]

_BUILTINS = ("exit", "quit")


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
        try:
            words: list[str] = shlex.split(text)
        except ValueError:
            return
        if text.endswith(" ") or not text:
            words.append("")
        current = words[-1]

        found = False

        if current.startswith("-"):
            for opt in GLOBAL_OPTS:
                if opt.startswith(current):
                    found = True
                    yield Completion(opt, start_position=-len(current))

        cmd_obj, _rem = self._find(words[:-1])
        if cmd_obj is None:
            for b in self._BUILTINS:
                if b.startswith(current):
                    found = True
                    yield Completion(b, start_position=-len(current))

        if cmd_obj is None:
            for key in self._cmd_map:
                if len(key) == 1 and key[0].startswith(current):
                    found = True
                    yield Completion(key[0], start_position=-len(current))
            return

        is_group = hasattr(cmd_obj, "registered_commands") or hasattr(
            cmd_obj, "registered_groups"
        )
        if is_group:
            names = [c.name for c in getattr(cmd_obj, "registered_commands", [])]
            names += [g.name for g in getattr(cmd_obj, "registered_groups", [])]
            for n in names:
                if n.startswith(current):
                    found = True
                    yield Completion(n, start_position=-len(current))

        if (not is_group) and hasattr(cmd_obj, "params"):
            for param in cmd_obj.params:
                for opt in (*param.opts, *(getattr(param, "secondary_opts", []) or [])):
                    if opt.startswith(current):
                        found = True
                        yield Completion(opt, start_position=-len(current))

        if "--help".startswith(current):
            found = True
            yield Completion("--help", start_position=-len(current))

        if not found:
            if (
                len(words) >= 3
                and words[0] == "config"
                and words[1] == "set"
                and words[2] == ""
            ):
                yield Completion("KEY=VALUE", display="KEY=VALUE", start_position=0)
            elif current == "":
                yield Completion("DUMMY", display="DUMMY", start_position=0)
