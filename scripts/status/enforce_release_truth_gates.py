#!/usr/bin/env python3
"""Enforce release truth gates for status claims and evidence artifacts."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
PARITY_MATRIX = ROOT / "artifacts" / "parity" / "command_parity_matrix.json"
PLUGIN_STATE = ROOT / "artifacts" / "status" / "plugin_state_report.json"
DOCS_AUDIT = ROOT / "artifacts" / "status" / "docs_audit.json"
TEST_AUDIT = ROOT / "artifacts" / "status" / "test_quality_audit.json"
CRATE_BOUNDARY_METRICS = ROOT / "artifacts" / "status" / "crate_boundary_metrics.json"
STATUS_DIR = ROOT / "artifacts" / "status"
README = ROOT / "README.md"

STATUS_FILES = [
    STATUS_DIR / "what_is_done.json",
    STATUS_DIR / "what_is_left.json",
    STATUS_DIR / "what_is_partial.json",
    STATUS_DIR / "what_is_deferred.json",
    STATUS_DIR / "what_is_intentionally_different.json",
]


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def read_text(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="ignore")


def has_claim(pattern: str, text: str) -> bool:
    return re.search(pattern, text, flags=re.IGNORECASE) is not None


def parity_stats() -> dict[str, int]:
    rows = read_json(PARITY_MATRIX).get("commands", [])
    out = {"complete": 0, "partial": 0, "missing": 0, "intentionally-different": 0}
    for row in rows:
        status = str(row.get("status", "missing"))
        if status in out:
            out[status] += 1
        else:
            out["missing"] += 1
    return out


def weak_test_count() -> int:
    rows = read_json(TEST_AUDIT).get("tests", [])
    count = 0
    for row in rows:
        score = int(row.get("shallow_score", 0))
        if score >= 5:
            count += 1
    return count


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    warnings: list[str] = []

    for required in STATUS_FILES:
        if not required.exists():
            failures.append(f"missing status artifact: {required.relative_to(ROOT)}")

    readme = read_text(README)
    public_claim_docs = [
        ROOT / "docs" / "HONEST_STATUS.md",
        ROOT / "docs" / "KNOWN_GAPS.md",
        ROOT / "docs" / "STABILITY_AND_BREAKAGE.md",
        ROOT / "docs" / "CONTRIBUTOR_ENGINEERING_RULES.md",
        ROOT / "docs" / "index.md",
    ]
    docs = "\n".join(read_text(path) for path in public_claim_docs if path.exists())
    claims_text = f"{readme}\n{docs}"

    stats = parity_stats()
    docs_audit = read_json(DOCS_AUDIT)
    plugin_state = read_json(PLUGIN_STATE)
    crate_metrics = read_json(CRATE_BOUNDARY_METRICS)

    if has_claim(r"feature\s+complete", claims_text):
        if stats["missing"] > 0 or stats["partial"] > 0:
            failures.append(
                f"feature-complete claim blocked: matrix has partial={stats['partial']} missing={stats['missing']}"
            )

    if has_claim(r"rust[-\s]*first\s+complete", claims_text):
        if stats["missing"] > 0 or stats["partial"] > 0:
            failures.append("rust-first-complete claim blocked: parity matrix not converged")

    if has_claim(r"plugin\s+system\s+complete", claims_text):
        partial = plugin_state.get("plugin_commands", {}).get("partial", [])
        if isinstance(partial, list) and partial:
            failures.append("plugin-system-complete claim blocked: plugin report still lists partial commands")

    if has_claim(r"docs\s+done", claims_text):
        markdown_count = int(docs_audit.get("markdown_count", 0))
        if markdown_count > int(docs_audit.get("target_long_form_docs_cap", 60)):
            failures.append(f"docs-done claim blocked: markdown_count={markdown_count} exceeds cap")

    if has_claim(r"tests\s+strong", claims_text):
        weak_count = weak_test_count()
        weak_threshold = 6
        if weak_count > weak_threshold:
            failures.append(
                f"tests-strong claim blocked: weak test count {weak_count} is above threshold {weak_threshold}"
            )

    boundary_rules = crate_metrics.get("rules", {})
    if not bool(boundary_rules.get("no_large_merge_until_parity_stronger", False)):
        failures.append("crate merge freeze rule missing in crate boundary metrics artifact")

    # Evidence rule for README: promotional quality claims require artifact references.
    if has_claim(r"\b(98%\+ coverage|1,800\+ tests|feature complete|production ready)\b", readme):
        if "artifacts/" not in readme and "docs/HONEST_STATUS.md" not in readme:
            failures.append("README claim is not evidence-backed with repository artifacts")
        else:
            warnings.append("README contains strong claims; keep evidence links current")

    for msg in warnings:
        print(f"TRUTH WARNING: {msg}")
    for msg in failures:
        print(f"TRUTH FAILURE: {msg}")

    if failures and args.enforce:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
