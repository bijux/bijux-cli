# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Parsing helpers for REPL input and command suggestions."""

from __future__ import annotations

from collections.abc import Iterator
from contextlib import suppress
import json
from pathlib import Path
import re
from typing import cast

from rapidfuzz import process as rf_process

_ansi_re = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
_semicolon_re = re.compile(
    r"""
    ;
    (?=(?:[^'"]|'[^']*'|"[^"]*")*$)
""",
    re.VERBOSE,
)


def _filter_control(text: str) -> str:
    """Removes ANSI control sequences from a string."""
    return _ansi_re.sub("", text)


def _split_segments(input_text: str) -> Iterator[str]:
    """Splits input text into individual, non-empty command segments."""
    clean = _filter_control(input_text)
    for ln in clean.splitlines():
        for part in _semicolon_re.split(ln):
            seg = part.strip()
            if seg:
                yield seg


def _known_commands() -> list[str]:
    """Loads the list of known CLI commands."""
    here = Path(__file__).resolve()
    for p in [here, *here.parents]:
        spec = p.parent / "spec.json"
        if spec.is_file():
            with suppress(Exception):
                data = json.loads(spec.read_text())
                cmds = data.get("commands")
                if isinstance(cmds, list):
                    return cmds
    return [
        "audit",
        "config",
        "dev",
        "docs",
        "doctor",
        "help",
        "history",
        "memory",
        "plugins",
        "repl",
        "sleep",
        "status",
        "version",
    ]


def _suggest(cmd: str) -> str | None:
    """Suggests a command based on fuzzy matching."""
    choices = _known_commands()
    if not choices:
        return None
    result = cast(tuple[str, int, object], rf_process.extractOne(cmd, choices))
    best, score, _ = result
    if score >= 60 and best != cmd:
        return f" Did you mean '{best}'?"
    return None
