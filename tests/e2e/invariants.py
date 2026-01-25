# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Shared E2E invariants for CLI behavior."""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass

from tests.e2e.harness import E2EHarness


@dataclass(frozen=True)
class E2EState:
    """Serializable state snapshot for E2E invariants."""

    config_text: str
    plugins: tuple[str, ...]
    history_text: str | None


def capture_state(h: E2EHarness) -> E2EState:
    """Capture configuration, plugins, and history state."""
    config_text = (
        h.config_path.read_text(encoding="utf-8") if h.config_path.exists() else ""
    )
    history_text = None
    plugins: tuple[str, ...] = ()
    if h.plugins_dir.exists():
        plugins = tuple(sorted(p.name for p in h.plugins_dir.iterdir() if p.is_dir()))
    return E2EState(config_text=config_text, plugins=plugins, history_text=history_text)


def assert_no_state_corruption(before: E2EState, after: E2EState) -> None:
    """Assert that state did not change across a failure path (ignoring history)."""
    assert before.config_text == after.config_text
    assert before.plugins == after.plugins


def assert_exit_code_stable(codes: Iterable[int]) -> None:
    """Assert that the same class of action returns stable exit codes."""
    codes = list(codes)
    assert codes, "expected at least one exit code"
    assert all(code == codes[0] for code in codes)


def assert_no_traceback(text: str) -> None:
    """Ensure CLI output does not contain a traceback."""
    assert "traceback" not in text.lower()


def assert_config_consistent(h: E2EHarness) -> None:
    """Ensure config file has unique keys and valid lines."""
    if not h.config_path.exists():
        return
    lines = [
        line for line in h.config_path.read_text(encoding="utf-8").splitlines() if line
    ]
    seen: set[str] = set()
    for line in lines:
        assert "=" in line, f"invalid config line: {line!r}"
        key, _value = line.split("=", 1)
        assert key.startswith("BIJUXCLI_"), f"invalid key prefix: {key!r}"
        assert key not in seen, f"duplicate config key: {key!r}"
        seen.add(key)


def assert_plugins_consistent(h: E2EHarness) -> None:
    """Ensure plugins directory contains only directories."""
    if not h.plugins_dir.exists():
        return
    for entry in h.plugins_dir.iterdir():
        if entry.name == ".bijux_install.lock":
            continue
        assert entry.is_dir(), f"unexpected plugin entry: {entry}"
