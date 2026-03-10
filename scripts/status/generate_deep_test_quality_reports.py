#!/usr/bin/env python3
"""Generate deep test-quality reports for commands/config/history/memory/diagnostics domains."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_ROOT = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface"

DOMAIN_RULES = {
    "commands": lambda rel: any(k in rel for k in ["command", "root", "cli_", "ported", "help"]),
    "config": lambda rel: "config" in rel,
    "history": lambda rel: "history" in rel,
    "memory": lambda rel: "memory" in rel,
    "diagnostics": lambda rel: any(k in rel for k in ["diagnostics", "doctor", "inspect", "dev_cli_output_contracts"]),
}


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")


def score(text: str) -> int:
    lower = text.lower()
    assert_count = text.count("assert!(") + text.count("assert_eq!(")
    has_failure = bool(re.search(r"failure|error|malformed|missing|invalid|usage", lower))
    has_determinism = "for _ in" in lower or "repeat" in lower or "repeated" in lower or "determin" in lower
    has_consistency = "consisten" in lower or "schema" in lower or "shape" in lower
    has_state = "corrupt" in lower or "rollback" in lower
    return assert_count + (3 if has_failure else 0) + (2 if has_determinism else 0) + (2 if has_consistency else 0) + (2 if has_state else 0)


def classify_missing_deep_cases(domain: str, tests: list[dict[str, Any]]) -> list[str]:
    merged = "\n".join((t["text"] for t in tests)).lower()
    required_by_domain = {
        "commands": [
            "unknown command usage",
            "deterministic repeated run",
            "stderr stdout separation",
        ],
        "config": [
            "rollback on invalid mutation",
            "corruption recovery",
            "precedence consistency",
        ],
        "history": [
            "malformed interleaving resilience",
            "deterministic ordering",
            "state doctor consistency",
        ],
        "memory": [
            "wrong type field handling",
            "missing state handling",
            "corruption diagnostics consistency",
        ],
        "diagnostics": [
            "findings order determinism",
            "schema consistency",
            "source of truth consistency",
        ],
    }
    cues = {
        "unknown command usage": ["unknown-command", "unknown command", "usage"],
        "deterministic repeated run": ["repeat", "repeated", "determin"],
        "stderr stdout separation": ["stderr", "stdout"],
        "rollback on invalid mutation": ["rollback", "invalid", "does not alter"],
        "corruption recovery": ["corrupt", "malformed", "recovery"],
        "precedence consistency": ["precedence", "source_precedence"],
        "malformed interleaving resilience": ["malformed", "interleav", "resilience"],
        "deterministic ordering": ["ordering", "determin"],
        "state doctor consistency": ["state-doctor", "doctor"],
        "wrong type field handling": ["wrong-type", "wrong type"],
        "missing state handling": ["missing", "count"],
        "corruption diagnostics consistency": ["corrupt", "doctor", "consisten"],
        "findings order determinism": ["findings", "issues", "determin"],
        "schema consistency": ["schema", "shape", "contracts"],
        "source of truth consistency": ["source", "routes", "registry", "env"],
    }

    missing = []
    for item in required_by_domain[domain]:
        if not any(cue in merged for cue in cues[item]):
            missing.append(item)
    return missing


def main() -> int:
    rows: list[dict[str, Any]] = []
    for path in sorted(TEST_ROOT.glob("*.rs")):
        rel = str(path.relative_to(ROOT)).replace("\\", "/")
        text = read(path)
        rows.append({"path": rel, "text": text, "score": score(text), "assert_count": text.count("assert!(") + text.count("assert_eq!(")})

    by_value: dict[str, Any] = {"generated_at": datetime.now(timezone.utc).isoformat(), "generator": "scripts/status/generate_deep_test_quality_reports.py", "domains": {}}
    missing_cases: dict[str, Any] = {"generated_at": by_value["generated_at"], "generator": by_value["generator"], "domains": {}}
    weak_replace: dict[str, Any] = {"generated_at": by_value["generated_at"], "generator": by_value["generator"], "domains": {}}

    for domain, predicate in DOMAIN_RULES.items():
        tests = [row for row in rows if predicate(row["path"].lower())]
        ranked = sorted(tests, key=lambda r: (-r["score"], -r["assert_count"], r["path"]))
        weakest = sorted(tests, key=lambda r: (r["score"], r["assert_count"], r["path"]))[:8]

        by_value["domains"][domain] = {
            "count": len(tests),
            "top_by_value": [{"path": t["path"], "value_score": t["score"]} for t in ranked[:20]],
        }
        missing_cases["domains"][domain] = classify_missing_deep_cases(domain, tests)
        weak_replace["domains"][domain] = [
            {"path": t["path"], "value_score": t["score"], "replacement_goal": "add failure-path or determinism proof"}
            for t in weakest
        ]

    contract = {
        "generated_at": by_value["generated_at"],
        "generator": by_value["generator"],
        "status": "frozen",
        "domains": ["commands", "config", "history", "memory", "diagnostics"],
        "rules": [
            "new command features require at least one deep failure-path or determinism test",
            "new diagnostics features require at least one consistency or shape test",
            "new stateful features require at least one corruption or rollback test",
        ],
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "deep_tests_by_value_report.json").write_text(json.dumps(by_value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (STATUS / "deep_missing_behavior_cases_report.json").write_text(json.dumps(missing_cases, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (STATUS / "deep_weak_tests_replacement_report.json").write_text(json.dumps(weak_replace, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (STATUS / "deep_test_first_domains_contract.json").write_text(json.dumps(contract, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("wrote artifacts/status/deep_tests_by_value_report.json")
    print("wrote artifacts/status/deep_missing_behavior_cases_report.json")
    print("wrote artifacts/status/deep_weak_tests_replacement_report.json")
    print("wrote artifacts/status/deep_test_first_domains_contract.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
