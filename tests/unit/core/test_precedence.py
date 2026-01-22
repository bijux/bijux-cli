# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Tests for flag precedence and logging semantics."""

from __future__ import annotations

from pathlib import Path

from bijux_cli.core.precedence import resolve_effective_config, resolve_output_flags

_TRUTH_TABLE = """| quiet | log-level flag | -v/-vv | --json | --color | effective log_level | include runtime | format | color |
|------:|----------------|-------:|-------:|--------:|--------------------:|----------------:|-------:|------:|
| false | info           | 0      | false  | auto    | info                | false           | json   | auto  |
| false | info           | 1      | false  | auto    | info                | true            | json   | auto  |
| false | warning        | 2      | false  | always  | warning             | true            | json   | always|
| false | error          | 0      | true   | never   | error               | false           | json   | never |
| true  | any            | any    | any    | any     | error               | false           | json   | any   |
"""


def test_resolve_output_flags_default() -> None:
    result = resolve_output_flags(
        quiet=False,
        verbose=False,
        pretty=False,
        log_level="info",
        color="auto",
    )
    assert result["log_level"] == "info"
    assert result["include_runtime"] is False
    assert result["pretty"] is False
    assert result["color"] == "auto"


def test_resolve_output_flags_verbose() -> None:
    result = resolve_output_flags(
        quiet=False,
        verbose=True,
        pretty=False,
        log_level="info",
        color="auto",
    )
    assert result["log_level"] == "info"
    assert result["include_runtime"] is True
    assert result["pretty"] is False


def test_resolve_output_flags_debug_overrides() -> None:
    result = resolve_output_flags(
        quiet=False,
        verbose=False,
        pretty=False,
        log_level="debug",
        color="always",
    )
    assert result["log_level"] == "debug"
    assert result["include_runtime"] is True
    assert result["pretty"] is True
    assert result["color"] == "always"


def test_resolve_output_flags_quiet_wins() -> None:
    result = resolve_output_flags(
        quiet=True,
        verbose=True,
        pretty=True,
        log_level="debug",
        color="auto",
    )
    assert result["log_level"] == "error"
    assert result["include_runtime"] is False
    assert result["pretty"] is True


def test_resolve_effective_config_json_forces_format() -> None:
    effective = resolve_effective_config(
        cli={"json": True, "color": "always"},
        env={},
        file={},
        defaults={"format": "yaml", "color": "auto"},
    )
    assert effective.fmt == "json"
    assert effective.color == "always"


def test_architecture_truth_table_matches_doc() -> None:
    table = _TRUTH_TABLE.strip()
    repo_root = Path(__file__).resolve().parents[3]
    doc = (repo_root / "docs/ARCHITECTURE.md").read_text(encoding="utf-8")
    assert table in doc
