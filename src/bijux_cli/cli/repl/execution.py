# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Execution helpers for the REPL."""

from __future__ import annotations

from contextlib import suppress
import json
import os
import shlex
import sys

from bijux_cli.cli.core.constants import OPT_FORMAT, OPT_QUIET, PRETTY_FLAGS
from bijux_cli.cli.repl.parsing import (
    _filter_control,
    _known_commands,
    _split_segments,
    _suggest,
)
from bijux_cli.core.enums import ExitCode, OutputFormat
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.runtime import run_command

_JSON_CMDS = {
    "audit",
    "doctor",
    "history",
    "memory",
    "plugins",
    "status",
    "version",
}


def _invoke(tokens: list[str], *, repl_quiet: bool) -> int:
    """Runs a single CLI command invocation within the REPL sandbox."""
    from importlib import import_module

    from typer.testing import CliRunner

    env = {**os.environ, "PS1": ""}

    head = tokens[0] if tokens else ""

    if head in _JSON_CMDS and not (set(PRETTY_FLAGS) | set(OPT_FORMAT)) & set(tokens):
        tokens.append("--no-pretty")

    if (
        head == "config"
        and len(tokens) > 1
        and tokens[1] == "list"
        and not (set(PRETTY_FLAGS) & set(tokens))
    ):
        tokens.append("--no-pretty")

    cli_root = import_module("bijux_cli.cli.root")
    app = getattr(cli_root, "build_app", None)
    typer_app = app() if callable(app) else cli_root.app
    result = CliRunner().invoke(typer_app, tokens, env=env)

    sub_quiet = any(t in OPT_QUIET for t in tokens)
    should_print = not repl_quiet and not sub_quiet

    if head == "history":
        with suppress(Exception):
            data = json.loads(result.stdout or "{}")
            if data.get("entries", []) == []:
                if should_print:
                    from bijux_cli.cli.core.command import resolve_serializer

                    pretty = (
                        resolve_serializer()
                        .dumps(data, fmt=OutputFormat.JSON, pretty=True)
                        .rstrip("\n")
                        + "\n"
                    )
                    sys.stdout.write(pretty)
                    sys.stderr.write(result.stderr or "")
                return result.exit_code

    if should_print:
        sys.stdout.write(result.stdout or "")
        sys.stderr.write(result.stderr or "")

    return result.exit_code


def _run_piped(repl_quiet: bool) -> None:
    """Processes piped input commands in non-interactive mode."""
    for raw_line in sys.stdin.read().splitlines():
        line = raw_line.rstrip()

        if not line or line.lstrip().startswith("#"):
            if not repl_quiet:
                from bijux_cli.cli.repl.ui import get_prompt

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
                    """Emits a JSON error for a `config` subcommand."""
                    if repl_quiet:
                        return

                    error_obj = {
                        "error": msg,
                        "code": 2,
                        "failure": failure,
                        "command": f"config {subcommand[0] if subcommand else ''}".strip(),
                        "format": "json",
                    }
                    from bijux_cli.cli.core.command import resolve_serializer

                    print(
                        resolve_serializer().dumps(
                            error_obj, fmt=OutputFormat.JSON, pretty=False
                        )
                    )

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


def run_repl_session(*, quiet: bool, stdin_isatty: bool) -> None:
    """Run the REPL in piped or interactive mode."""
    if quiet or not stdin_isatty:
        _run_piped(quiet)
    else:
        from bijux_cli.cli.repl.ui import _run_interactive

        run_command(_run_interactive)
