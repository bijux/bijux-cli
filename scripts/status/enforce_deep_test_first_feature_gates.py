#!/usr/bin/env python3
"""Enforce deep-test-first gates for commands, diagnostics, and stateful features."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def changed_test_files(base_rev: str) -> list[Path]:
    cmd = ["git", "diff", "--name-only", f"{base_rev}..HEAD", "--", "crates/*/tests/*.rs"]
    out = subprocess.run(cmd, cwd=ROOT, check=False, text=True, capture_output=True)
    files = []
    for line in out.stdout.splitlines():
        p = ROOT / line.strip()
        if p.exists() and p.suffix == ".rs":
            files.append(p)
    return files


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore").lower()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    parser.add_argument("--base-rev", default="HEAD~1")
    args = parser.parse_args()

    contract_file = STATUS / "deep_test_first_domains_contract.json"
    if not contract_file.exists():
        print("DEEP TEST-FIRST FAILURE: missing artifacts/status/deep_test_first_domains_contract.json")
        return 1 if args.enforce else 0

    contract = json.loads(contract_file.read_text(encoding="utf-8"))
    if contract.get("status") != "frozen":
        print("DEEP TEST-FIRST FAILURE: deep test-first contract is not frozen")
        return 1 if args.enforce else 0

    failures: list[str] = []
    for path in changed_test_files(args.base_rev):
        rel = str(path.relative_to(ROOT)).replace("\\", "/")
        text = read(path)

        is_command = any(k in rel for k in ["command", "root", "cli_", "help", "ported"])
        is_diagnostics = any(k in rel for k in ["diagnostics", "doctor", "inspect"])
        is_stateful = any(k in rel for k in ["config", "history", "memory", "state"])

        if is_command and not any(k in text for k in ["failure", "error", "unknown", "determin", "repeat"]):
            failures.append(f"command test lacks deep failure/determinism evidence: {rel}")

        if is_diagnostics and not any(k in text for k in ["consisten", "schema", "shape", "contract"]):
            failures.append(f"diagnostics test lacks consistency/shape evidence: {rel}")

        if is_stateful and not any(k in text for k in ["corrupt", "rollback", "malformed", "missing"]):
            failures.append(f"stateful test lacks corruption/rollback evidence: {rel}")

    if failures:
        print("DEEP TEST-FIRST GATE FAILURES:")
        for item in failures:
            print(f" - {item}")
        return 1 if args.enforce else 0

    print("Deep test-first feature gates passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
