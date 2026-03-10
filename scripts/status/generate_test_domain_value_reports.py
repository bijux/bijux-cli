#!/usr/bin/env python3
"""Generate plugin/flag/determinism test-value and missing-abuse reports."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
AUDIT = ROOT / "artifacts" / "status" / "test_quality_audit.json"
OUT = ROOT / "artifacts" / "status"


def load_audit() -> list[dict]:
    if not AUDIT.exists():
        return []
    payload = json.loads(AUDIT.read_text(encoding="utf-8"))
    tests = payload.get("tests", []) if isinstance(payload, dict) else []
    return [row for row in tests if isinstance(row, dict)]


def read_text(rel_path: str) -> str:
    path = ROOT / rel_path
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="ignore")


def score_value(row: dict, text: str) -> int:
    score = 0
    if row.get("has_failure_terms"):
        score += 3
    if row.get("has_exit_assertion"):
        score += 2
    if row.get("has_output_assertion"):
        score += 2
    asserts = int(row.get("assert_count", 0) or 0)
    score += min(asserts, 6)
    if "rollback" in text.lower() or "corrupt" in text.lower() or "failure" in text.lower():
        score += 2
    if "snapshot" in text.lower() and asserts <= 2:
        score -= 2
    score -= int(row.get("shallow_score", 0) or 0)
    return score


def domain_rows(rows: list[dict], predicate) -> list[dict]:
    out = []
    for row in rows:
        rel = str(row.get("path", ""))
        if not predicate(rel):
            continue
        text = read_text(rel)
        entry = dict(row)
        entry["value_score"] = score_value(row, text)
        entry["weak_reasons"] = [
            reason
            for reason, cond in [
                ("missing failure-path assertions", not row.get("has_failure_terms")),
                ("missing exit-code assertions", not row.get("has_exit_assertion")),
                ("missing stream assertions", not row.get("has_output_assertion")),
                ("shallow structure", int(row.get("shallow_score", 0) or 0) >= 4),
            ]
            if cond
        ]
        out.append(entry)
    out.sort(key=lambda r: (r["value_score"], r.get("assert_count", 0), r.get("path", "")))
    return out


def missing_plugin_abuse(tests: list[dict]) -> list[str]:
    merged = "\n".join(read_text(str(t.get("path", ""))).lower() for t in tests)
    required = [
        "namespace collision",
        "alias collision",
        "install rollback",
        "uninstall rollback",
        "missing entrypoint",
        "registry corruption",
        "permission denied",
    ]
    return [item for item in required if item.split()[0] not in merged]


def missing_flag_abuse(tests: list[dict]) -> list[str]:
    merged = "\n".join(read_text(str(t.get("path", ""))).lower() for t in tests)
    required = [
        "conflicting pretty flags",
        "conflicting color flags",
        "repeated format flags",
        "mixed global local ordering",
        "missing format value",
        "unknown global flag",
    ]
    return [item for item in required if item.split()[0] not in merged]


def missing_deterministic_proofs(tests: list[dict]) -> list[str]:
    missing = []
    for row in tests:
        rel = str(row.get("path", ""))
        text = read_text(rel).lower()
        if "for _ in" not in text and "repeat" not in text and "across runs" not in text:
            missing.append(rel)
    return missing


def write_json(name: str, payload: dict) -> None:
    path = OUT / name
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")


def main() -> int:
    rows = load_audit()

    plugin = domain_rows(rows, lambda rel: "plugin" in rel and rel.endswith(".rs"))
    flag = domain_rows(rows, lambda rel: "flag" in rel or "parser" in rel)
    determinism = domain_rows(rows, lambda rel: "deterministic" in rel)

    now = datetime.now(timezone.utc).isoformat()

    write_json(
        "plugin_tests_by_value.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "domain": "plugin",
            "tests": plugin,
            "weak_tests": [row for row in plugin if row["value_score"] <= 2],
        },
    )
    write_json(
        "flag_tests_by_value.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "domain": "flag",
            "tests": flag,
            "weak_tests": [row for row in flag if row["value_score"] <= 2],
        },
    )
    write_json(
        "determinism_tests_by_value.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "domain": "determinism",
            "tests": determinism,
            "weak_tests": [row for row in determinism if row["value_score"] <= 2],
        },
    )

    write_json(
        "missing_plugin_abuse_cases.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "cases": missing_plugin_abuse(plugin),
        },
    )
    write_json(
        "missing_flag_abuse_cases.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "cases": missing_flag_abuse(flag),
        },
    )
    write_json(
        "missing_deterministic_proof_cases.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "cases": missing_deterministic_proofs(determinism),
        },
    )
    write_json(
        "test_first_domains_contract.json",
        {
            "generated_at": now,
            "generator": "scripts/status/generate_test_domain_value_reports.py",
            "domains": ["plugin", "flag", "determinism"],
            "policy": [
                "new plugin features require failure-path coverage",
                "new flag features require precedence or conflict coverage",
                "new determinism claims require repeated-run proof",
                "new tests in these domains must carry a test_type tag",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
