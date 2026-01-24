# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""UI helpers for the REPL prompt and interactive loop."""

from __future__ import annotations

from contextlib import suppress
import os
import shlex
import signal
import sys
from types import FrameType

from prompt_toolkit.formatted_text import ANSI
from prompt_toolkit.key_binding.key_processor import KeyPressEvent

from bijux_cli.cli.core.constants import (
    ENV_BIN,
    ENV_HISTORY_FILE,
    ENV_NO_COLOR,
    ENV_TEST_MODE,
)
from bijux_cli.cli.repl.completion import CommandCompleter
from bijux_cli.cli.repl.execution import _invoke
from bijux_cli.cli.repl.parsing import _known_commands, _split_segments, _suggest
from bijux_cli.core.enums import ExitCode, OutputFormat
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError


def _exit_on_signal(_signum: int, _frame: FrameType | None = None) -> None:
    """Exits the process cleanly when a watched signal is received."""
    raise ExitIntentError(
        ExitIntent(
            code=ExitCode.SUCCESS,
            stream=None,
            payload=None,
            fmt=OutputFormat.JSON,
            pretty=False,
            show_traceback=False,
        )
    )


def get_prompt() -> str | ANSI:
    """Returns the REPL prompt string."""
    if os.environ.get(ENV_TEST_MODE) == "1" or os.environ.get(ENV_NO_COLOR) == "1":
        return "bijux> "
    return ANSI("\x1b[36mbijux> \x1b[0m")


async def _run_interactive() -> None:
    """Starts the interactive REPL session."""
    from importlib import import_module
    from pathlib import Path
    import subprocess  # nosec B404

    from prompt_toolkit import PromptSession
    from prompt_toolkit.auto_suggest import AutoSuggestFromHistory
    from prompt_toolkit.buffer import Buffer
    from prompt_toolkit.completion import Completion
    from prompt_toolkit.history import FileHistory
    from prompt_toolkit.key_binding import KeyBindings
    from prompt_toolkit.output import ColorDepth

    cli_mod = import_module("bijux_cli.cli.root")
    build_app = getattr(cli_mod, "build_app", None)
    app = build_app() if callable(build_app) else cli_mod.app

    kb = KeyBindings()

    @kb.add("tab")
    def _(event: KeyPressEvent) -> None:
        """Handles Tab key presses for completion."""
        buf = event.app.current_buffer
        if buf.complete_state:
            buf.complete_next()
        else:
            buf.start_completion(select_first=True)

    @kb.add("enter")
    def _(event: KeyPressEvent) -> None:
        """Handles Enter key presses to submit or accept completions."""
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
        ENV_HISTORY_FILE,
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

    cli_bin = os.environ.get(ENV_BIN) or sys.argv[0]

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


def register_signal_handlers() -> None:
    """Register REPL signal handlers for a clean exit."""
    for sig in (
        signal.SIGINT,
        signal.SIGTERM,
        signal.SIGHUP,
        signal.SIGQUIT,
        signal.SIGUSR1,
    ):
        with suppress(Exception):
            signal.signal(sig, _exit_on_signal)
