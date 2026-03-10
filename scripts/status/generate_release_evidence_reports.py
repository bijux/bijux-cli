#!/usr/bin/env python3
"""Generate release evidence bundle, release truth report, and release status manifest."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
PARITY = ROOT / "artifacts" / "parity"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def exists_status(paths: list[Path]) -> list[dict[str, Any]]:
    return [
        {
            "path": rel(path),
            "exists": path.exists(),
        }
        for path in paths
    ]


def main() -> None:
    generated_at = stable_generated_at()

    parity_matrix = PARITY / "command_parity_matrix.json"
    runtime_identity = STATUS / "runtime_unity_report.json"
    package_health = STATUS / "package_health_report.json"
    plugin_hardening = STATUS / "plugin_lifecycle_failure_injection_report.json"
    state_hardening = STATUS / "state_resilience_summary.json"
    performance = STATUS / "performance_report.json"
    known_gaps = ROOT / "docs" / "KNOWN_GAPS.md"

    release_evidence_paths = [
        parity_matrix,
        runtime_identity,
        package_health,
        plugin_hardening,
        state_hardening,
        performance,
        known_gaps,
    ]

    evidence = exists_status(release_evidence_paths)
    missing = [item["path"] for item in evidence if not item["exists"]]

    parity = read_json(parity_matrix)
    rows = parity.get("commands", []) if isinstance(parity, dict) else []
    partial = [r.get("command") for r in rows if isinstance(r, dict) and r.get("status") == "partial"]
    missing_cmd = [r.get("command") for r in rows if isinstance(r, dict) and r.get("status") == "missing"]

    scripts_audit = read_json(STATUS / "script_only_behaviors.json")
    docs_audit = read_json(STATUS / "docs_audit.json")
    test_audit = read_json(STATUS / "test_quality_audit.json")

    stale_scripts = scripts_audit.get("scripts", []) if isinstance(scripts_audit, dict) else []
    weak_tests = []
    for row in test_audit.get("tests", []) if isinstance(test_audit, dict) else []:
        if isinstance(row, dict) and int(row.get("shallow_score", 0)) >= 5:
            weak_tests.append(row.get("path", ""))

    release_bundle = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_release_evidence_reports.py",
        "scope": "release evidence bundle",
        "status": "complete" if not missing else "partial",
        "tasks": [561, 562, 563, 564, 565, 566, 567, 568],
        "evidence": evidence,
        "missing": missing,
    }

    manifest = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_release_evidence_reports.py",
        "scope": "release status manifest",
        "status": "ready" if not missing else "blocked",
        "tasks": [579],
        "checks": {
            "missing_evidence": missing,
            "parity_partial_count": len(partial),
            "parity_missing_count": len(missing_cmd),
            "stale_scripts_outside_dev_cli": len(stale_scripts),
            "docs_markdown_count": int(docs_audit.get("markdown_count", 0)) if isinstance(docs_audit, dict) else 0,
            "weak_tests_count": len(weak_tests),
        },
        "review_steps": [
            "review intentionally different behaviors",
            "review unresolved partial commands",
            "review stale scripts outside dev cli",
            "review stale docs from docs audit",
            "review weak tests from test audit",
        ],
    }

    truth_lines = [
        "Release Truth Report",
        "",
        f"status: {manifest['status']}",
        f"missing_evidence: {len(missing)}",
        f"parity_partial: {len(partial)}",
        f"parity_missing: {len(missing_cmd)}",
        f"stale_scripts_outside_dev_cli: {len(stale_scripts)}",
        f"weak_tests: {len(weak_tests)}",
    ]
    if missing:
        truth_lines.append("")
        truth_lines.append("missing evidence:")
        truth_lines.extend([f"- {item}" for item in missing])
    truth_lines.append("")
    truth_lines.append("no hype: release claims must match evidence and manifest status")

    truth = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_release_evidence_reports.py",
        "scope": "release truth",
        "status": manifest["status"],
        "tasks": [578, 580],
        "summary": {
            "missing_evidence": len(missing),
            "parity_partial": len(partial),
            "parity_missing": len(missing_cmd),
            "weak_tests": len(weak_tests),
        },
        "claim_policy": "release claims are evidence-only",
    }

    write_json(STATUS / "release_evidence_bundle.json", release_bundle)
    write_json(STATUS / "release_status_manifest.json", manifest)
    write_json(STATUS / "release_truth_report.json", truth)
    write_text(STATUS / "release_truth_report.txt", "\n".join(truth_lines) + "\n")

    print("wrote artifacts/status/release_evidence_bundle.json")
    print("wrote artifacts/status/release_status_manifest.json")
    print("wrote artifacts/status/release_truth_report.json")
    print("wrote artifacts/status/release_truth_report.txt")


if __name__ == "__main__":
    main()
