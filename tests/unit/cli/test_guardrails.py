# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Architecture tests for CLI guardrails."""

from __future__ import annotations

from pathlib import Path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def test_no_raw_cli_flags_in_commands() -> None:
    """Command modules must not embed raw option/env strings."""
    root = Path("src/bijux_cli/cli/commands")
    banned = (
        "--quiet",
        "--format",
        "--log-level",
        "--color",
        "--pretty",
        "--no-pretty",
        "NO_COLOR",
        "BIJUXCLI_",
    )
    offenders: list[str] = []
    for path in root.rglob("*.py"):
        text = _read_text(path)
        if any(token in text for token in banned):
            offenders.append(str(path))
    assert offenders == [], f"Raw option/env strings found: {offenders}"


def test_parse_global_flags_only_in_root() -> None:
    """Global flag parsing must live only in cli/root.py."""
    root = Path("src/bijux_cli")
    offenders: list[str] = []
    for path in root.rglob("*.py"):
        if path.name == "flags.py":
            continue
        if path.as_posix().endswith("cli/root.py"):
            continue
        if path.as_posix().endswith("core/intent.py"):
            continue
        if "parse_global_flags" in _read_text(path):
            offenders.append(str(path))
    assert offenders == [], f"parse_global_flags used outside cli/root.py: {offenders}"
