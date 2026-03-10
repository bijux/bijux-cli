#!/usr/bin/env python3
"""Enforce bijux-dev-cli canonical control-plane ownership declarations."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    release_truth = (
        ROOT / "docs" / "architecture" / "dev_cli_control_plane_release_truth.md"
    ).read_text(encoding="utf-8")
    lib_rs = (ROOT / "crates" / "bijux-dev-cli" / "src" / "lib.rs").read_text(encoding="utf-8")

    failures: list[str] = []
    if "canonical control-plane crate" not in release_truth:
        failures.append("release truth note must declare canonical control-plane ownership")
    if "`bijux dev cli status`" not in release_truth:
        failures.append("release truth note must declare status as default dashboard")
    if "`bijux dev cli parity`" not in release_truth:
        failures.append("release truth note must declare parity as default migration dashboard")
    if "Maintainer control-plane modules for `bijux dev cli ...` workflows." not in lib_rs:
        failures.append("bijux-dev-cli crate docs must keep explicit maintainer control-plane scope")

    if failures:
        for failure in failures:
            print(f"canonical-control-plane-failure: {failure}")
        return 1
    print("dev cli canonical control-plane policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

