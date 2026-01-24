# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Architecture guardrails for macOS artifact files."""

from __future__ import annotations

from pathlib import Path


def test_no_macos_dot_underscore_files_in_src() -> None:
    """Fail if macOS dot-underscore files appear under src/."""
    root = Path(__file__).resolve().parents[3]
    src = root / "src"
    offenders = sorted(p for p in src.rglob("._*") if p.is_file())
    assert not offenders, f"Found macOS artifact files: {offenders}"
