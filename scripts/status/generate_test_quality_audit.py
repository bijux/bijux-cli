#!/usr/bin/env python3
"""Generate test inventory and quality classification artifacts."""

from __future__ import annotations

import json
import re
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "test_quality_audit.json"
WEAKEST_OUT = ROOT / "artifacts" / "status" / "top_20_weakest_tests.json"
MISSING_FAILURE_OUT = ROOT / "artifacts" / "status" / "top_20_missing_failure_cases.json"
MISSING_PARITY_OUT = ROOT / "artifacts" / "status" / "top_20_missing_parity_cases.json"

CATEGORY_PATTERNS = [
    ("parity", ["parity", "python"]),
    ("snapshot", ["snapshot", "snapshots"]),
    ("perf", ["benchmark", "performance", "latency"]),
    ("resilience", ["malformed", "corrupt", "failure", "rollback", "missing"]),
    ("property", ["property", "proptest", "hypothesis"]),
    ("regression", ["regression", "golden", "compatibility"]),
]


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT)).replace("\\", "/")


def list_test_files() -> list[Path]:
    return sorted(p for p in ROOT.rglob("*.rs") if "tests" in p.parts and "target" not in p.parts)


def classify(path: Path, text: str) -> str:
    lower = f"{path.name} {text[:4000]}".lower()
    for category, patterns in CATEGORY_PATTERNS:
        if any(p in lower for p in patterns):
            return category
    if shallow_score(text) >= 4:
        return "weak"
    return "regression"


def shallow_score(text: str) -> int:
    score = 0
    if text.count("assert_eq!") + text.count("assert!(") <= 2:
        score += 2
    if "stderr" not in text and "exit_code" not in text and "status.code" not in text:
        score += 2
    if "malformed" not in text and "missing" not in text and "failure" not in text and "error" not in text:
        score += 2
    return score


def collect_missing_scenarios(files: list[Path]) -> dict[str, list[str]]:
    merged = "\n".join(p.read_text(encoding="utf-8", errors="ignore").lower() for p in files)

    missing_failure = []
    for scenario in [
        "unknown command path",
        "invalid config mutation input",
        "plugin lifecycle rollback",
        "filesystem permission failure",
        "partial write recovery",
        "corrupt registry handling",
        "malformed history line",
        "malformed memory state",
        "unsupported output format",
        "runtime identity drift",
        "path override precedence conflict",
        "env override precedence conflict",
        "quiet mode with failures",
        "trace mode with failures",
        "help suggestions on unknown command",
        "binary bridge parity regression",
        "exit mapping drift",
        "stderr routing drift",
        "duplicate plugin install handling",
        "plugin namespace collision",
    ]:
        key = scenario.split()[0]
        if key not in merged:
            missing_failure.append(scenario)

    missing_parity = []
    for scenario in [
        "dev cli inventory parity",
        "dev cli status parity",
        "docs-prune-plan parity",
        "package-health parity",
        "crate-health parity",
        "scripts-audit parity",
        "snapshots-audit parity",
        "fixture-audit parity",
        "runtime-identity parity",
        "config export parity",
        "config load parity",
        "history parity",
        "memory parity",
        "help parity nested routes",
        "plugin inspect parity",
        "plugin list parity",
        "doctor parity",
        "paths parity",
        "inspect parity",
        "self-test parity",
    ]:
        if scenario.split()[0] not in merged:
            missing_parity.append(scenario)

    missing_packaging = []
    for scenario in [
        "pip install invocation parity",
        "pipx invocation parity",
        "python -m invocation parity",
        "wheel metadata consistency",
        "entrypoint naming stability",
        "cargo and pip conflict detection",
        "multi-path binary shadowing",
        "installer marker drift",
        "package script mismatch",
        "bridge load failure fallback",
        "bridge import api stability",
        "version sync between runtimes",
        "help sync between runtimes",
        "error envelope sync between runtimes",
        "plugin operation sync between runtimes",
        "release artifact integrity",
        "release note evidence gate",
        "docs claim evidence gate",
        "api schema packaging gate",
        "homebrew channel parity placeholder",
    ]:
        if scenario.split()[0] not in merged:
            missing_packaging.append(scenario)

    return {
        "top_20_missing_failure_cases": missing_failure[:20],
        "top_20_missing_parity_cases": missing_parity[:20],
        "top_20_missing_packaging_scenarios": missing_packaging[:20],
    }


def main() -> int:
    files = list_test_files()
    rows = []
    category_counts = Counter()

    for path in files:
        text = path.read_text(encoding="utf-8", errors="ignore")
        category = classify(path, text)
        score = shallow_score(text)
        category_counts[category] += 1
        rows.append(
            {
                "path": rel(path),
                "category": category,
                "assert_count": text.count("assert!(") + text.count("assert_eq!("),
                "has_failure_terms": bool(re.search(r"failure|error|malformed|missing|rollback", text, re.I)),
                "has_exit_assertion": "exit_code" in text or "status.code" in text,
                "has_output_assertion": "stdout" in text or "stderr" in text,
                "shallow_score": score,
            }
        )

    weakest = sorted(rows, key=lambda r: (-r["shallow_score"], r["assert_count"], r["path"]))[:20]
    missing = collect_missing_scenarios(files)

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_test_quality_audit.py",
        "test_file_count": len(rows),
        "category_counts": dict(sorted(category_counts.items())),
        "tests": rows,
        "top_20_weakest_tests": weakest,
        **missing,
        "quality_rules": [
            "new commands require at least one failure-path test",
            "stateful commands require at least one corruption or rollback test",
            "parser changes require malformed-input coverage",
            "plugin changes require namespace or rollback coverage",
            "output changes require snapshot or diff coverage",
            "install changes require ambiguity or path-failure coverage",
            "no vanity test growth",
        ],
        "flaky_tests": {
            "label": "flaky",
            "policy": "flaky tests must be explicitly tagged in ci metadata and tracked as debt",
            "tagging_source": "artifacts/status/flaky_tests.json",
        },
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    WEAKEST_OUT.write_text(
        json.dumps(
            {
                "generated_at": report["generated_at"],
                "generator": report["generator"],
                "tests": weakest,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    MISSING_FAILURE_OUT.write_text(
        json.dumps(
            {
                "generated_at": report["generated_at"],
                "generator": report["generator"],
                "cases": missing["top_20_missing_failure_cases"],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    MISSING_PARITY_OUT.write_text(
        json.dumps(
            {
                "generated_at": report["generated_at"],
                "generator": report["generator"],
                "cases": missing["top_20_missing_parity_cases"],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    print(f"wrote {OUT.relative_to(ROOT)}")
    print(f"wrote {WEAKEST_OUT.relative_to(ROOT)}")
    print(f"wrote {MISSING_FAILURE_OUT.relative_to(ROOT)}")
    print(f"wrote {MISSING_PARITY_OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
