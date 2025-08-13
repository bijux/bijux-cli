# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the interactive Read-Eval-Print Loop (REPL) for the Bijux CLI.

This module provides a rich, interactive shell for executing Bijux CLI commands.
It enhances the user experience with features like persistent command history,
context-aware tab-completion, and a colorized prompt. Users can chain multiple
commands on a single line using semicolons. The REPL can also operate in a
non-interactive mode to process commands piped from stdin.

The REPL itself operates in a human-readable format. When executing commands,
it respects global flags like `--format` or `--quiet` for those specific
invocations.

Exit Codes:
    * `0`: The REPL session was exited cleanly (e.g., via `exit`, `quit`,
      Ctrl+D, or a caught signal).
    * `2`: An invalid flag was provided to the `repl` command itself
      (e.g., `--format=json`).
"""

from __future__ import annotations

import asyncio
from collections.abc import Iterator
from contextlib import suppress
import json
import os
from pathlib import Path
import re
import shlex
import signal
import sys
from types import FrameType
from typing import Any

from prompt_toolkit.buffer import Buffer
from prompt_toolkit.completion import CompleteEvent, Completer, Completion
from prompt_toolkit.document import Document
from prompt_toolkit.formatted_text import ANSI
from prompt_toolkit.key_binding.key_processor import KeyPressEvent
from rapidfuzz import process as rf_process
import typer

from bijux_cli.commands.utilities import emit_error_and_exit, validate_common_flags
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT_HELP,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)

GLOBAL_OPTS = [
    "-q",
    "--quiet",
    "-v",
    "--verbose",
    "-f",
    "--format",
    "--pretty",
    "--no-pretty",
    "-d",
    "--debug",
    "-h",
    "--help",
]

repl_app = typer.Typer(
    name="repl",
    help="Starts an interactive shell with history and tab-completion.",
    add_completion=False,
)

_ansi_re = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")


def _filter_control(text: str) -> str:
    """Removes ANSI control sequences from a string.

    Args:
        text (str): The input string that may contain ANSI escape codes.

    Returns:
        str: A cleaned version of the string with ANSI codes removed.
    """
    return _ansi_re.sub("", text)


_semicolon_re = re.compile(
    r"""
    ;
    (?=(?:[^'"]|'[^']*'|"[^"]*")*$)
""",
    re.VERBOSE,
)


def _exit_on_signal(_signum: int, _frame: FrameType | None = None) -> None:
    """Exits the process cleanly when a watched signal is received.

    Args:
        _signum (int): The signal number that triggered the handler (unused).
        _frame (FrameType | None): The current stack frame (unused).
    """
    sys.exit(0)


def _split_segments(input_text: str) -> Iterator[str]:
    """Splits input text into individual, non-empty command segments.

    Commands are separated by newlines or by semicolons that are not inside
    quotes.

    Args:
        input_text (str): The raw input text.

    Yields:
        str: A cleaned, non-empty command segment.
    """
    clean = _filter_control(input_text)
    for ln in clean.splitlines():
        for part in _semicolon_re.split(ln):
            seg = part.strip()
            if seg:
                yield seg


def _known_commands() -> list[str]:
    """Loads the list of known CLI commands.

    It attempts to load commands from a `spec.json` file located in the
    project structure. If the file is not found or is invalid, it returns a
    hard-coded default list of commands.

    Returns:
        list[str]: A list of known command names.
    """
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
    """Suggests a command based on fuzzy matching.

    Args:
        cmd (str): The user-provided command input to evaluate.

    Returns:
        str | None: A hint message (e.g., " Did you mean 'status'?") if a
            sufficiently similar command is found, otherwise None.
    """
    choices = _known_commands()
    best, score, _ = rf_process.extractOne(cmd, choices)
    if score >= 60 and best != cmd:
        return f" Did you mean '{best}'?"
    return None


def _invoke(tokens: list[str], *, repl_quiet: bool) -> int:
    """Runs a single CLI command invocation within the REPL sandbox.

    It prepares the arguments, invokes the command via `CliRunner`, and handles
    the printing of output based on the quiet flags.

    Args:
        tokens (list[str]): The shell-split tokens for the command.
        repl_quiet (bool): If True, all stdout/stderr from the invocation
            is suppressed.

    Returns:
        int: The exit code returned by the command invocation.
    """
    from typer.testing import CliRunner

    from bijux_cli.cli import app as _root_app

    env = {**os.environ, "PS1": ""}

    head = tokens[0] if tokens else ""

    if head in _JSON_CMDS and not {"--pretty", "--no-pretty", "-f", "--format"} & set(
        tokens
    ):
        tokens.append("--no-pretty")

    if (
        head == "config"
        and len(tokens) > 1
        and tokens[1] == "list"
        and "--no-pretty" not in tokens
        and "--pretty" not in tokens
    ):
        tokens.append("--no-pretty")

    result = CliRunner().invoke(_root_app, tokens, env=env)

    sub_quiet = any(t in ("-q", "--quiet") for t in tokens)
    should_print = not repl_quiet and not sub_quiet

    if head == "history":
        with suppress(Exception):
            data = json.loads(result.stdout or "{}")
            if data.get("entries", []) == []:
                if should_print:
                    pretty = json.dumps(data, indent=2) + "\n"
                    sys.stdout.write(pretty)
                    sys.stderr.write(result.stderr or "")
                return result.exit_code

    if should_print:
        sys.stdout.write(result.stdout or "")
        sys.stderr.write(result.stderr or "")

    return result.exit_code


def _run_piped(repl_quiet: bool) -> None:
    """Processes piped input commands in non-interactive mode.

    Reads from stdin, splits commands, and executes them sequentially. This
    mode is activated when stdin is not a TTY or the `--quiet` flag is used.

    Args:
        repl_quiet (bool): If True, suppresses prompts and error messages.

    Returns:
        None:
    """
    for raw_line in sys.stdin.read().splitlines():
        line = raw_line.rstrip()

        if not line or line.lstrip().startswith("#"):
            if not repl_quiet:
                sys.stderr.write(_filter_control(str(get_prompt())) + "\n")
                sys.stderr.flush()
            continue

        if line.lstrip().startswith(";"):
            bad = line.lstrip(";").strip()
            hint = _suggest(bad)
            msg = f"No such command '{bad}'." + (hint or "")
            if not repl_quiet:
                print(msg, file=sys.stderr)
            continue

        for seg in _split_segments(line):
            seg = seg.strip()
            if not seg or seg.startswith("#"):
                continue

            lo = seg.lower()
            if lo in {"exit", "quit"}:
                sys.exit(0)

            if seg == "docs":
                if not repl_quiet:
                    print("Available topics: …")
                continue
            if seg.startswith("docs "):
                if not repl_quiet:
                    print(seg.split(None, 1)[1])
                continue

            if seg.startswith("-"):
                bad = seg.lstrip("-")
                hint = _suggest(bad)
                msg = f"No such command '{bad}'." + (hint or "")
                if not repl_quiet:
                    print(msg, file=sys.stderr)
                continue

            try:
                tokens = shlex.split(seg)
            except ValueError:
                continue
            if not tokens:
                continue

            head = tokens[0]

            if head == "config":
                sub = tokens[1:]

                def _emit(
                    msg: str,
                    failure: str,
                    subcommand: list[str] = sub,
                ) -> None:
                    """Emits a JSON error for a `config` subcommand.

                    Args:
                        msg (str): The error message.
                        failure (str): A short failure code (e.g., "parse").
                        subcommand (list[str]): The subcommand tokens.
                    """
                    if repl_quiet:
                        return

                    error_obj = {
                        "error": msg,
                        "code": 2,
                        "failure": failure,
                        "command": f"config {subcommand[0] if subcommand else ''}".strip(),
                        "format": "json",
                    }
                    print(json.dumps(error_obj))

                if not sub:
                    pass
                elif sub[0] == "set" and len(sub) == 1:
                    _emit("Missing argument: KEY=VALUE required", "missing_argument")
                    continue
                elif sub[0] in {"get", "unset"} and len(sub) == 1:
                    _emit("Missing argument: key required", "missing_argument")
                    continue

            if head not in _known_commands():
                hint = _suggest(head)
                msg = f"No such command '{head}'."
                if hint:
                    msg += hint
                if not repl_quiet:
                    print(msg, file=sys.stderr)
                continue
            else:
                _invoke(tokens, repl_quiet=repl_quiet)

    sys.exit(0)


def get_prompt() -> str | ANSI:
    """Returns the REPL prompt string.

    The prompt is styled with ANSI colors unless `NO_COLOR` or a test mode
    environment variable is set.

    Returns:
        str | ANSI: The prompt string, which may include ANSI color codes.
    """
    if os.environ.get("BIJUXCLI_TEST_MODE") == "1" or os.environ.get("NO_COLOR") == "1":
        return "bijux> "
    return ANSI("\x1b[36mbijux> \x1b[0m")


_JSON_CMDS = {
    "audit",
    "doctor",
    "history",
    "memory",
    "plugins",
    "status",
    "version",
}
_BUILTINS = ("exit", "quit")


class CommandCompleter(Completer):
    """Provides context-aware tab-completion for the REPL."""

    def __init__(self, main_app: typer.Typer) -> None:
        """Initializes the completer.

        Args:
            main_app (typer.Typer): The root Typer application whose commands
                and options will be used for completion suggestions.
        """
        self.main_app = main_app
        self._cmd_map = self._collect(main_app)
        self._BUILTINS = _BUILTINS

    def _collect(
        self,
        app: typer.Typer,
        path: list[str] | None = None,
    ) -> dict[tuple[str, ...], Any]:
        """Recursively collects all commands from a Typer application.

        Args:
            app (typer.Typer): The Typer application to scan.
            path (list[str] | None): The accumulated command path for recursion.

        Returns:
            dict[tuple[str, ...], Any]: A mapping from command-path tuples to
                the corresponding Typer app or command object.
        """
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
        """Finds the best-matching command or group for the given tokens.

        Args:
            words (list[str]): The list of tokens from the input buffer.

        Returns:
            tuple[Any | None, list[str]]: A tuple containing the matched
                command/group object and the list of remaining tokens.
        """
        for i in range(len(words), 0, -1):
            key = tuple(words[:i])
            if key in self._cmd_map:
                return self._cmd_map[key], words[i:]
        return None, words

    def get_completions(  # pyright: ignore[reportIncompatibleMethodOverride]
        self,
        document: Document,
        _complete_event: CompleteEvent,
    ) -> Iterator[Completion]:
        """Yields completion suggestions for the current input.

        Args:
            document (Document): The current `prompt_toolkit` document.
            _complete_event (CompleteEvent): The completion event (unused).

        Yields:
            Completion: A `prompt_toolkit` `Completion` object.
        """
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

        if isinstance(cmd_obj, typer.Typer):
            names = [c.name for c in getattr(cmd_obj, "registered_commands", [])]
            names += [g.name for g in getattr(cmd_obj, "registered_groups", [])]
            for n in names:
                if n.startswith(current):
                    found = True
                    yield Completion(n, start_position=-len(current))

        if not isinstance(cmd_obj, typer.Typer) and hasattr(cmd_obj, "params"):
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


async def _run_interactive() -> None:
    """Starts the interactive REPL session.

    This function configures and runs a `prompt_toolkit` session, providing
    an interactive shell for the user. It handles user input asynchronously.

    Returns:
        None:
    """
    from importlib import import_module
    import os
    from pathlib import Path
    import shlex
    import subprocess  # nosec B404
    import sys

    from prompt_toolkit import PromptSession
    from prompt_toolkit.auto_suggest import AutoSuggestFromHistory
    from prompt_toolkit.history import FileHistory
    from prompt_toolkit.key_binding import KeyBindings
    from prompt_toolkit.output import ColorDepth

    cli_mod = import_module("bijux_cli.cli")
    app = cli_mod.build_app()

    kb = KeyBindings()

    @kb.add("tab")
    def _(event: KeyPressEvent) -> None:
        """Handles Tab key presses for completion.

        Args:
            event (KeyPressEvent): The `prompt_toolkit` key press event.
        """
        buf = event.app.current_buffer
        if buf.complete_state:
            buf.complete_next()
        else:
            buf.start_completion(select_first=True)

    @kb.add("enter")
    def _(event: KeyPressEvent) -> None:
        """Handles Enter key presses to submit or accept completions.

        Args:
            event (KeyPressEvent): The `prompt_toolkit` key press event.
        """
        buf: Buffer = event.app.current_buffer
        state = buf.complete_state
        if state:
            comp: Completion | None = state.current_completion
            if comp:
                buf.apply_completion(comp)
            buf.complete_state = None
        else:
            buf.validate_and_handle()

    history_file = os.environ.get(
        "BIJUXCLI_HISTORY_FILE",
        str(Path.home() / ".bijux" / ".repl_history"),
    )

    session: PromptSession[str] = PromptSession(
        get_prompt(),
        history=FileHistory(history_file),
        completer=CommandCompleter(app),
        auto_suggest=AutoSuggestFromHistory(),
        color_depth=ColorDepth.DEPTH_1_BIT,
        enable_history_search=True,
        complete_while_typing=False,
        key_bindings=kb,
    )

    cli_bin = os.environ.get("BIJUXCLI_BIN") or sys.argv[0]

    while True:
        try:
            line = await session.prompt_async()
        except (EOFError, KeyboardInterrupt):
            print("\nExiting REPL.")
            return

        for seg in _split_segments(line):
            lower = seg.lower()
            if lower in ("exit", "quit"):
                print("Exiting REPL.")
                return
            if not seg.strip() or seg.startswith("#"):
                continue

            try:
                tokens = shlex.split(seg)
            except ValueError:
                continue

            head = tokens[0]
            if seg == "docs":
                print("Available topics: ...")
                continue
            if seg.startswith("docs "):
                print(seg.split(None, 1)[1])
                continue
            if seg == "memory list":
                subprocess.run(  # noqa: S603 # nosec B603
                    [cli_bin, *tokens], env=os.environ
                )
                continue

            if head not in _known_commands():
                hint = _suggest(head)
                msg = f"No such command '{head}'."
                if hint:
                    msg += hint
                print(msg, file=sys.stderr)
                continue

            _invoke(tokens, repl_quiet=False)


@repl_app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("human", "-f", "--format", help=HELP_FORMAT_HELP),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Defines the entrypoint for the `bijux repl` command.

    This function initializes the REPL environment. It validates flags, sets
    up signal handlers for clean shutdown, and dispatches to either the
    non-interactive (piped) mode or the interactive async prompt loop.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, forces non-interactive mode and suppresses
            prompts and command output.
        verbose (bool): If True, enables verbose output for subcommands.
        fmt (str): The desired output format. Only "human" is supported for
            the REPL itself.
        pretty (bool): If True, enables pretty-printing for subcommands.
        debug (bool): If True, enables debug diagnostics for subcommands.

    Returns:
        None:
    """
    if ctx.invoked_subcommand:
        return

    command = "repl"
    effective_include_runtime = (verbose or debug) and not quiet

    fmt_lower = fmt.strip().lower()

    if fmt_lower != "human":
        validate_common_flags(
            fmt_lower,
            command,
            quiet,
            include_runtime=effective_include_runtime,
        )
        emit_error_and_exit(
            "REPL only supports human format.",
            code=2,
            failure="format",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    for sig in (
        signal.SIGINT,
        signal.SIGTERM,
        signal.SIGHUP,
        signal.SIGQUIT,
        signal.SIGUSR1,
    ):
        with suppress(Exception):
            signal.signal(sig, _exit_on_signal)

    if quiet or not sys.stdin.isatty():
        _run_piped(quiet)
    else:
        try:
            asyncio.get_event_loop()
        except RuntimeError:
            asyncio.set_event_loop(asyncio.new_event_loop())

        asyncio.run(_run_interactive())


if __name__ == "__main__":
    repl_app()
