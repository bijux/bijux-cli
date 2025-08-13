# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi
# ruff: noqa: S101, S607

"""End-to-end tests for `bijux repl` command suggestion."""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from pathlib import Path
import re

import pytest

from tests.e2e.conftest import run_cli


def _mk_env(tmp_path: Path) -> dict[str, str]:
    return {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
        "BIJUXCLI_TEST_MODE": "1",
    }


def _run_repl(script: str, env: dict[str, str]) -> str:
    """Run the repl with `script`, return combined stdout+stderr."""
    proc = run_cli(["repl"], env=env, input_data=script)
    assert proc.returncode == 0, proc
    return proc.stdout + proc.stderr


def _any_contains(haystack: str, needles: Iterable[str]) -> bool:
    """Case‑insensitive substring check for any needle."""
    hay = haystack.lower()
    return any(n.lower() in hay for n in needles)


def _matches_regex(haystack: str, patterns: Sequence[str]) -> bool:
    """Case‑insensitive regex search for any pattern."""
    return any(re.search(p, haystack, flags=re.IGNORECASE) for p in patterns)


@dataclass(frozen=True)
class SuggestCase:
    """A single contract for typo/suggestion behaviour."""

    line: str
    must_contain: Sequence[str] = ()
    any_of: Sequence[str] = ()
    regex_any: Sequence[str] = ()
    must_not: Sequence[str] = ()


CASES: list[SuggestCase] = [
    SuggestCase("cnofig", any_of=("No such command", "Did you mean")),
    SuggestCase("co", any_of=("Ambiguous", "Did you mean", "No such command")),
    SuggestCase("ConFiG", any_of=("config", "Did you mean", "No such command")),
    SuggestCase(
        "versoin",
        any_of=("version", "Did you mean", "No such command"),
    ),
    SuggestCase("ver", any_of=("version", "Did you mean", "No such command")),
    SuggestCase("exittt", any_of=("exit", "Did you mean", "No such command")),
    SuggestCase("ex", any_of=("exit", "Did you mean", "No such command")),
    SuggestCase("config s", any_of=("set", "Did you mean", "No such command")),
    SuggestCase("config se", any_of=("Ambiguous", "Did you mean", "No such command")),
    SuggestCase("config sett", any_of=("set", "Did you mean", "No such command")),
    SuggestCase("config srt", any_of=("No such command", "Did you mean")),
    SuggestCase("plugins i", any_of=("install", "Did you mean", "No such command")),
    SuggestCase(
        "plugins installt", any_of=("install", "Did you mean", "No such command")
    ),
    SuggestCase("dev d", any_of=("di", "Did you mean", "No such command")),
    SuggestCase("config exportt", any_of=("export", "Did you mean", "No such command")),
    SuggestCase("config; co", any_of=("config", "Did you mean", "No such command")),
    SuggestCase(";config", any_of=("config", "Did you mean", "No such command")),
    SuggestCase("config;;set", any_of=("set", "Did you mean", "No such command")),
    SuggestCase("--f", any_of=("No such command", "Did you mean")),
    SuggestCase("--for", any_of=("format", "Did you mean", "No such command")),
    SuggestCase("-d", any_of=(), must_not=("traceback",)),
    SuggestCase("nonsense_command", any_of=("No such command", "Did you mean")),
    SuggestCase("konfig", any_of=("No such command", "Did you mean")),
    SuggestCase("cnofgi srt foo=bar", any_of=("No such command", "Did you mean")),
    SuggestCase(
        "   config     set   foo=bar   ", must_contain=("foo",), must_not=("traceback",)
    ),
    SuggestCase("   ", any_of=("bijux>",)),
    SuggestCase("", any_of=("bijux>",)),
    SuggestCase("# just a comment", any_of=("bijux>",)),
    SuggestCase("\n", any_of=("bijux>",)),
    SuggestCase("helpme", any_of=("help", "Did you mean", "No such command")),
    SuggestCase(
        "hel",
        any_of=("help", "Did you mean", "No such command"),
    ),
    SuggestCase("historyy", any_of=("history", "Did you mean", "No such command")),
    SuggestCase("hist", any_of=("history", "Did you mean", "No such command")),
]


@pytest.mark.parametrize("case", CASES, ids=lambda c: c.line or "<blank>")
def test_repl_suggestion_contract(case: SuggestCase, tmp_path: Path) -> None:
    env = _mk_env(tmp_path)
    out = _run_repl(f"{case.line}\nexit\n", env)

    for token in case.must_contain:
        assert token.lower() in out.lower(), f"Expected '{token}' in output\n{out}"

    if case.any_of:
        assert _any_contains(out, case.any_of), (
            f"Expected any of {case.any_of} in output\n{out}"
        )

    if case.regex_any:
        assert _matches_regex(out, case.regex_any), (
            f"Expected any regex {case.regex_any} to match output\n{out}"
        )

    for forbidden in case.must_not:
        assert forbidden.lower() not in out.lower(), (
            f"Unexpected '{forbidden}' in output\n{out}"
        )
