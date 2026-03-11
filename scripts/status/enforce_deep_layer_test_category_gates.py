#!/usr/bin/env python3
"""Enforce deep-layer category gates."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

REQUIRED = [
    STATUS / "plugin_tests_by_category.json",
    STATUS / "flag_tests_by_category.json",
    STATUS / "determinism_tests_by_category.json",
    STATUS / "command_tests_by_category.json",
    STATUS / "config_tests_by_category.json",
    STATUS / "history_tests_by_category.json",
    STATUS / "memory_tests_by_category.json",
    STATUS / "diagnostics_tests_by_category.json",
    STATUS / "repl_tests_by_category.json",
    STATUS / "bridge_tests_by_category.json",
    STATUS / "top_10_weakest_deep_layer_tests.json",
    STATUS / "deep_layer_weak_test_actions.json",
    STATUS / "deep_layer_test_coverage_artifact.json",
    STATUS / "deep_layer_behaviors_contract.json",
]


def changed_files(base_rev: str, pattern: str) -> list[Path]:
    out = subprocess.run(
        ["git", "diff", "--name-only", f"{base_rev}..HEAD", "--", pattern],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    files = []
    for line in out.stdout.splitlines():
        p = ROOT / line.strip()
        if p.exists():
            files.append(p)
    return files


def load_lower(paths: list[Path]) -> str:
    chunks = []
    for path in paths:
        chunks.append(path.read_text(encoding="utf-8", errors="ignore").lower())
    return "\n".join(chunks)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-rev", default="HEAD~1")
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []

    missing = [p for p in REQUIRED if not p.exists()]
    if missing:
        failures.append("missing artifacts: " + ", ".join(str(p.relative_to(ROOT)) for p in missing))

    coverage = json.loads((STATUS / "deep_layer_test_coverage_artifact.json").read_text(encoding="utf-8")) if (
        STATUS / "deep_layer_test_coverage_artifact.json"
    ).exists() else {}
    contract = json.loads((STATUS / "deep_layer_behaviors_contract.json").read_text(encoding="utf-8")) if (
        STATUS / "deep_layer_behaviors_contract.json"
    ).exists() else {}

    if contract.get("status") != "frozen":
        failures.append("deep-layer behaviors contract is not frozen")

    # 395: every new deep-layer behavior change touches at least one relevant test category
    src = changed_files(args.base_rev, "crates/*/src/*.rs")
    test_files = changed_files(args.base_rev, "crates/*/tests/*.rs")
    if src and not test_files:
        failures.append("source changes detected without any test updates for deep-layer categories")

    test_text = load_lower(test_files) if test_files else ""

    # 396: new stateful features add corruption or rollback coverage
    state_src = [p for p in src if any(k in str(p).lower() for k in ["config", "history", "memory", "state", "plugin"])]
    if state_src and not any(k in test_text for k in ["corrupt", "rollback", "malformed", "broken"]):
        failures.append("stateful source changes lack corruption/rollback test evidence")

    # 397: new cross-surface behavior adds equivalence coverage
    cross_src = [p for p in src if any(k in str(p).lower() for k in ["python", "bridge", "repl", "parity", "cross"])]
    if cross_src and not any(k in test_text for k in ["parity", "equival", "agree", "bridge", "repl", "cross-surface"]):
        failures.append("cross-surface source changes lack equivalence coverage evidence")

    # 398: new determinism claims add repeated-run proof
    diff = subprocess.run(
        ["git", "diff", f"{args.base_rev}..HEAD", "--", "crates/*/src/*.rs"],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    ).stdout.lower()
    determinism_claim = any(token in diff for token in ["determin", "stable", "ordering", "byte-stable"])
    if determinism_claim and not any(k in test_text for k in ["for _ in", "repeat", "repeated", "determin"]):
        failures.append("determinism-related source claims lack repeated-run proof in tests")

    if coverage and not coverage.get("top_10_weakest"):
        failures.append("top_10 weakest deep-layer tests report is empty")

    if failures:
        print("DEEP-LAYER CATEGORY GATE FAILURES:")
        for item in failures:
            print(f" - {item}")
        return 1 if args.enforce else 0

    print("deep-layer category gates passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
