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
PARITY_DIR = ROOT / "artifacts" / "parity"
README = ROOT / "README.md"

STATUS_FILES = [
    STATUS_DIR / "what_is_done.json",
    STATUS_DIR / "what_is_left.json",
    STATUS_DIR / "what_is_partial.json",
    STATUS_DIR / "what_is_deferred.json",
    STATUS_DIR / "what_is_intentionally_different.json",
    STATUS_DIR / "what_is_unproven.json",
    STATUS_DIR / "install_neutrality_report.json",
    STATUS_DIR / "active_runtime_report.json",
    STATUS_DIR / "package_health_report.json",
    STATUS_DIR / "install_health_report.json",
    STATUS_DIR / "command_family_closure_report.json",
    STATUS_DIR / "command_family_closure_report.txt",
    STATUS_DIR / "command_family_partial_area_acceptance.json",
    STATUS_DIR / "config_closure_report.json",
    STATUS_DIR / "plugins_closure_report.json",
    STATUS_DIR / "history_closure_report.json",
    STATUS_DIR / "memory_closure_report.json",
    STATUS_DIR / "diagnostics_closure_report.json",
    STATUS_DIR / "repl_shared_law_closure_report.json",
    STATUS_DIR / "cross_surface_consistency_artifact.json",
    STATUS_DIR / "cross_surface_drift_artifact.json",
    STATUS_DIR / "cross_surface_consistency_contract.json",
    STATUS_DIR / "simplification_deletion_artifact.json",
    STATUS_DIR / "candidate_merge_later_report.json",
    STATUS_DIR / "candidate_keep_separate_report.json",
    STATUS_DIR / "release_evidence_bundle.json",
    STATUS_DIR / "release_status_manifest.json",
    STATUS_DIR / "release_truth_report.json",
    STATUS_DIR / "release_truth_report.txt",
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
    out = {"complete": 0, "partial": 0, "missing": 0, "different-by-decision": 0}
    for row in rows:
        status = str(row.get("status", "missing"))
        if status in out:
            out[status] += 1
        elif status == "intentionally-different":
            out["different-by-decision"] += 1
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
        ROOT / "docs" / "WHAT_WE_WONT_CLAIM.md",
        ROOT / "docs" / "STABILITY_AND_BREAKAGE.md",
        ROOT / "docs" / "CONTRIBUTOR_ENGINEERING_RULES.md",
        ROOT / "docs" / "MAINTAINER_MILESTONE_CHECKLIST.md",
        ROOT / "docs" / "index.md",
    ]
    docs = "\n".join(read_text(path) for path in public_claim_docs if path.exists())
    claims_text = f"{readme}\n{docs}"

    stats = parity_stats()
    docs_audit = read_json(DOCS_AUDIT)
    plugin_state = read_json(PLUGIN_STATE)
    crate_metrics = read_json(CRATE_BOUNDARY_METRICS)
    runtime_unity = read_json(STATUS_DIR / "runtime_unity_report.json")
    parity_dashboard = read_json(PARITY_DIR / "parity_dashboard.json")
    command_family_closure = read_json(STATUS_DIR / "command_family_closure_report.json")
    partial_area_acceptance = read_json(STATUS_DIR / "command_family_partial_area_acceptance.json")
    cross_surface_consistency = read_json(STATUS_DIR / "cross_surface_consistency_artifact.json")
    cross_surface_drift = read_json(STATUS_DIR / "cross_surface_drift_artifact.json")
    release_bundle = read_json(STATUS_DIR / "release_evidence_bundle.json")
    release_manifest = read_json(STATUS_DIR / "release_status_manifest.json")
    release_truth = read_json(STATUS_DIR / "release_truth_report.json")
    compatibility_debt = read_json(STATUS_DIR / "compatibility_debt_trend_report.json")

    if has_claim(r"feature\s+complete", claims_text):
        if stats["missing"] > 0 or stats["partial"] > 0:
            failures.append(
                f"feature-complete claim blocked: matrix has partial={stats['partial']} missing={stats['missing']}"
            )

    if has_claim(r"rust[-\s]*first\s+complete", claims_text):
        if stats["missing"] > 0 or stats["partial"] > 0:
            failures.append("rust-first-complete claim blocked: parity matrix not converged")
        elif not runtime_unity.get("ok", False):
            failures.append("rust-first-complete claim blocked: runtime unity report is not healthy")

    if has_claim(r"plugin\s+system\s+complete", claims_text):
        partial = plugin_state.get("plugin_commands", {}).get("partial", [])
        if isinstance(partial, list) and partial:
            failures.append("plugin-system-complete claim blocked: plugin report still lists partial commands")
        if not plugin_state.get("overlap_parity_tests"):
            failures.append("plugin-system-complete claim blocked: plugin parity evidence is missing")
        if not plugin_state.get("remaining_gaps") == []:
            failures.append("plugin-system-complete claim blocked: plugin remaining gaps are not empty")

    if has_claim(r"docs\s+(done|cleaned\s*up)", claims_text):
        if not docs_audit:
            failures.append("docs-cleaned-up claim blocked: docs audit evidence missing")
        markdown_count = int(docs_audit.get("markdown_count", 0))
        if markdown_count <= 0:
            failures.append("docs-cleaned-up claim blocked: docs audit does not include markdown inventory")
        markdown_count = int(docs_audit.get("markdown_count", 0))
        if markdown_count > int(docs_audit.get("target_long_form_docs_cap", 60)):
            failures.append(f"docs-done claim blocked: markdown_count={markdown_count} exceeds cap")

    if has_claim(r"tests\s+strong", claims_text):
        if not TEST_AUDIT.exists():
            failures.append("tests-strong claim blocked: test quality audit evidence missing")
        weak_count = weak_test_count()
        weak_threshold = 6
        if weak_count > weak_threshold:
            failures.append(
                f"tests-strong claim blocked: weak test count {weak_count} is above threshold {weak_threshold}"
            )

    # Explicit claim laws from release truth requirements.
    if has_claim(r"equal\s+to\s+v?0\.2\.0", claims_text):
        failures.append("equal-to-v0.2.0 claim blocked: explicit parity-equivalence claims are not allowed")

    if has_claim(r"better\s+than\s+v?0\.2\.0", claims_text):
        failures.append("better-than-v0.2.0 claim blocked: superiority claims are not allowed")

    if has_claim(r"migration\s+complete", claims_text):
        trend_points = compatibility_debt.get("trend_points", [])
        unresolved = compatibility_debt.get("open_debt", [])
        if not isinstance(trend_points, list) or len(trend_points) == 0:
            failures.append(
                "migration-complete claim blocked: compatibility debt trend evidence is missing"
            )
        if isinstance(unresolved, list) and unresolved:
            failures.append(
                "migration-complete claim blocked: compatibility debt report still has open debt"
            )

    boundary_rules = crate_metrics.get("rules", {})
    if not bool(boundary_rules.get("no_large_merge_until_parity_stronger", False)):
        failures.append("crate merge freeze rule missing in crate boundary metrics artifact")

    if not parity_dashboard:
        failures.append("release review blocked: missing artifacts/parity/parity_dashboard.json")
    else:
        coverage = parity_dashboard.get("summary", {}).get("coverage", {})
        if int(coverage.get("parity_tests", 0)) <= 0:
            failures.append("release review blocked: parity dashboard reports zero parity_tests")

    if not command_family_closure:
        failures.append("release review blocked: missing command_family_closure_report evidence")
    else:
        reports = command_family_closure.get("reports", {})
        partial_areas = [
            name
            for name, payload in reports.items()
            if isinstance(payload, dict) and str(payload.get("status", "")) != "complete"
        ]
        accepted = partial_area_acceptance.get("accepted_areas", []) if isinstance(partial_area_acceptance, dict) else []
        if partial_areas and not isinstance(accepted, list):
            failures.append("partial area acceptance must list accepted_areas")
        elif partial_areas:
            missing_acceptance = sorted(set(partial_areas) - set(str(item) for item in accepted))
            if missing_acceptance:
                failures.append(
                    "release review blocked: partial areas require explicit acceptance: "
                    + ", ".join(missing_acceptance)
                )

    if not cross_surface_consistency or not cross_surface_drift:
        failures.append("release review blocked: missing cross-surface consistency evidence")
    else:
        covered_drift = [
            item
            for item in cross_surface_drift.get("drift_items", [])
            if isinstance(item, dict) and str(item.get("coverage_class", "partial")) == "covered"
        ]
        if covered_drift:
            failures.append(
                "release review blocked: covered cross-surface drift exists for "
                + ", ".join(str(item.get("todo", "?")) for item in covered_drift)
            )

    if not release_bundle or not release_manifest or not release_truth:
        failures.append("release review blocked: missing release truth bundle artifacts")
    else:
        if str(release_manifest.get("status", "blocked")) != "ready":
            failures.append("release review blocked: release status manifest is not ready")
        if str(release_truth.get("status", "blocked")) != str(release_manifest.get("status", "blocked")):
            failures.append("release review blocked: release truth status disagrees with release manifest")

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
