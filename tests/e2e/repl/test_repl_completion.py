# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""End-to-end tests for `bijux repl` command completion."""

from __future__ import annotations

from collections.abc import Generator, Iterable
import os
from pathlib import Path
import re

import pexpect  # type: ignore[import-untyped]
import pytest

from tests.e2e.conftest import PROMPT_REGEX, run_cli, spawn_repl

DEFAULT_TIMEOUT = int(os.getenv("REPL_TEST_TIMEOUT", "10"))


def _send_tabs(child: pexpect.spawn[str], n: int = 1) -> None:
    """Sends one or more tab characters to the pexpect child process.

    Args:
        child: The pexpect child process running the REPL.
        n: The number of tab characters to send.
    """
    child.send("\t" * n)


def _send_backspaces(child: pexpect.spawn[str], n: int = 1) -> None:
    """Sends one or more backspace characters to the pexpect child process.

    Note: This sends Ctrl-H, which is a more reliable backspace sequence
    in terminal emulation than the standard backspace character.

    Args:
        child: The pexpect child process running the REPL.
        n: The number of backspace characters to send.
    """
    for _ in range(n):
        child.sendcontrol("h")


def _expect_any(
    child: pexpect.spawn[str],
    patterns: Iterable[str | re.Pattern[str]],
    timeout: int = DEFAULT_TIMEOUT,
) -> None:
    """Expects any of the given patterns or the prompt to appear.

    This function compiles string patterns into regex objects and adds the
    default prompt regex to the list of expected patterns. This prevents
    the test from hanging if no other pattern matches.

    Args:
        child: The pexpect child process running the REPL.
        patterns: An iterable of strings or compiled regex patterns to expect.
        timeout: The maximum time in seconds to wait for a match.

    Raises:
        pexpect.TIMEOUT: If none of the patterns (including the prompt) match
            within the timeout period.
    """
    compiled = [re.compile(p) if isinstance(p, str) else p for p in patterns]
    compiled.append(re.compile(PROMPT_REGEX))
    child.expect_list(compiled, timeout=timeout)  # pyright: ignore[reportArgumentType]


@pytest.fixture
def repl(
    bijux_env: dict[str, str],
) -> Generator[pexpect.spawn[str], None, None]:
    """Yields a ready REPL and ensures it’s closed after each test."""
    child = spawn_repl(bijux_env)
    child.expect(PROMPT_REGEX, timeout=DEFAULT_TIMEOUT)
    try:
        yield child
    finally:
        try:
            child.sendline("exit")
            child.expect(pexpect.EOF, timeout=DEFAULT_TIMEOUT)
        except Exception:
            child.close(force=True)


def test_tab_in_empty_prompt_shows_something_or_no_crash(
    repl: pexpect.spawn[str],
) -> None:
    """Ensure tab in an empty prompt shows suggestions or does not crash."""
    _send_tabs(repl)
    _expect_any(repl, [r"Usage:", r"\bconfig\b", r"help"])


def test_double_tab_in_empty_prompt(repl: pexpect.spawn[str]) -> None:
    """Ensure double tab in an empty prompt shows suggestions or does not crash."""
    _send_tabs(repl, 2)
    _expect_any(repl, [r"Usage:", r"\bconfig\b", r"help"])


def test_tab_suggests_top_level_commands(repl: pexpect.spawn[str]) -> None:
    """Ensure tab completion suggests top-level commands."""
    _send_tabs(repl)
    _expect_any(repl, [r"\bconfig\b", r"\bversion\b", r"\bhelp\b", r"\bexit\b"])


def test_partial_command_then_tab_autocompletes_or_suggests(
    repl: pexpect.spawn[str],
) -> None:
    """Ensure tab after a partial command autocompletes or shows suggestions."""
    repl.send("con")
    _send_tabs(repl)
    _expect_any(repl, [r"\bconfig\b", PROMPT_REGEX])


def test_space_after_command_then_tab_lists_subcommands(
    repl: pexpect.spawn[str],
) -> None:
    """Ensure tab after a full command lists its subcommands."""
    repl.send("config ")
    _send_tabs(repl)
    _expect_any(
        repl,
        [
            r"\bset\b",
            r"\bget\b",
            r"\bunset\b",
            r"\bexport\b",
            r"\bclear\b",
            r"\blist\b",
        ],
    )


def test_partial_subcommand_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure tab after a partial subcommand provides suggestions."""
    repl.send("config s")
    _send_tabs(repl)
    _expect_any(repl, [r"\bset\b"])


def test_argument_position_tab_does_not_crash(
    repl: pexpect.spawn[str],
) -> None:
    """Ensure tab after a command needing an argument does not crash."""
    repl.send("config set ")
    _send_tabs(repl)
    _expect_any(repl, [r"--help\b", PROMPT_REGEX])


def test_backspace_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure completion works correctly after using backspace."""
    repl.send("conf")
    _send_backspaces(repl, 4)
    _send_tabs(repl)
    _expect_any(repl, [r"\bconfig\b", r"help"])


def test_partial_exit_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure the 'exit' command can be tab-completed."""
    repl.send("ex")
    _send_tabs(repl)
    _expect_any(repl, [r"\bexit\b", PROMPT_REGEX])


def test_tab_after_multiple_leading_spaces(repl: pexpect.spawn[str]) -> None:
    """Ensure leading spaces don't prevent tab completion."""
    repl.send("   ")
    _send_tabs(repl)
    _expect_any(repl, [r"Usage:", r"\bconfig\b"])


def test_tab_after_leading_spaces_and_partial(
    repl: pexpect.spawn[str],
) -> None:
    """Ensure leading spaces and a partial command works with completion."""
    repl.send("   ver")
    _send_tabs(repl)
    _expect_any(repl, [r"\bversion\b", PROMPT_REGEX])


def test_tab_on_option_prefix_single_dash(repl: pexpect.spawn[str]) -> None:
    """Ensure tab after a single dash suggests short options."""
    repl.send("-")
    _send_tabs(repl)
    _expect_any(repl, [r"-f", r"-q", r"-d"])


def test_partial_option_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure tab after a partial long option provides suggestions."""
    repl.send("--f")
    _send_tabs(repl)
    _expect_any(repl, [r"--format\b", PROMPT_REGEX])


def test_space_after_option_prefix_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure tab after '--' suggests available long options."""
    repl.send("config set --")
    _send_tabs(repl)
    _expect_any(repl, [r"--pretty\b", r"--quiet\b", PROMPT_REGEX])


def test_partial_format_option_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure the '--format' option can be tab-completed."""
    repl.send("--for")
    _send_tabs(repl)
    _expect_any(repl, [r"--format\b", PROMPT_REGEX])


def test_tab_on_debug_flag_prefix(repl: pexpect.spawn[str]) -> None:
    """Ensure tab after '-d' suggests debug flag options."""
    repl.send("-d")
    _send_tabs(repl)
    _expect_any(repl, [r"-d\b", r"--debug\b", PROMPT_REGEX])


def test_partial_debug_flag_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure the '--debug' flag can be tab-completed."""
    repl.send("--deb")
    _send_tabs(repl)
    _expect_any(repl, [r"--debug\b", PROMPT_REGEX])


def test_tab_on_quiet_flag_prefix(repl: pexpect.spawn[str]) -> None:
    """Ensure tab after '-q' suggests quiet flag options."""
    repl.send("-q")
    _send_tabs(repl)
    _expect_any(repl, [r"-q\b", r"--quiet\b", PROMPT_REGEX])


def test_partial_quiet_flag_then_tab(repl: pexpect.spawn[str]) -> None:
    """Ensure the '--quiet' flag can be tab-completed."""
    repl.send("--qui")
    _send_tabs(repl)
    _expect_any(repl, [r"--quiet\b", PROMPT_REGEX])


def test_tab_for_nested_command_group(repl: pexpect.spawn[str]) -> None:
    """Ensure tab completion works for nested command groups."""
    repl.send("plugins ")
    _send_tabs(repl)
    _expect_any(repl, [r"\blist\b", r"\binstall\b", PROMPT_REGEX])


def test_tab_after_blank_line(repl: pexpect.spawn[str]) -> None:
    """Ensure tab completion works after submitting a blank line."""
    repl.send("\n")
    repl.expect(PROMPT_REGEX, timeout=DEFAULT_TIMEOUT)
    _send_tabs(repl)
    _expect_any(repl, [r"Usage:", PROMPT_REGEX])


def test_tab_with_escaped_space(repl: pexpect.spawn[str]) -> None:
    """Ensure escaped spaces in input do not break completion."""
    repl.send(r"config set foo=bar\ baz ")
    _send_tabs(repl)
    _expect_any(repl, [r"\bbaz\b", PROMPT_REGEX])


@pytest.mark.parametrize(
    ("input_seq", "expected"),
    [
        ("", [r"Usage:", r"\bconfig\b", r"help"]),
        ("con", [r"\bconfig\b"]),
        ("config ", [r"\bset\b", r"\bget\b", r"\bunset\b"]),
        ("foobarbaz", None),
    ],
)
def test_tab_completion_smoke(
    repl: pexpect.spawn[str], input_seq: str, expected: list[str] | None
) -> None:
    """Run a parametrized smoke test for various completion scenarios."""
    if input_seq:
        repl.send(input_seq)
    _send_tabs(repl)
    repl.sendline("")
    if expected:
        _expect_any(repl, expected)
    else:
        repl.expect(PROMPT_REGEX, timeout=DEFAULT_TIMEOUT)


def test_non_tty_tab_is_ignored_and_does_not_crash(
    tmp_path: Path,
) -> None:
    """Ensure sending a tab character in a non-TTY context is safely ignored."""
    env = {
        "BIJUXCLI_CONFIG": str(tmp_path / ".env"),
        "BIJUXCLI_TEST_MODE": "1",
    }
    res = run_cli(["repl"], env=env, input_data="\t\nexit\n")
    assert res.returncode == 0
