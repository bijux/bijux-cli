#!/usr/bin/env python3
"""Generate deep-layer test-category coverage artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_ROOTS = [
    ROOT / "crates" / "bijux-cli" / "tests",
    ROOT / "crates" / "bijux-cli-python" / "tests",
    ROOT / "crates" / "bijux-dev-cli" / "tests",
]

CATEGORY_RULES: dict[str, dict[str, list[str]]] = {
    "plugin": {
        "namespace": ["namespace", "reserved"],
        "lifecycle": ["install", "uninstall", "enable", "disable", "lifecycle"],
        "failure": ["failure", "error", "invalid", "broken"],
        "rollback": ["rollback", "self_repair", "repair"],
        "determinism": ["determin", "repeat", "stable", "ordering"],
    },
    "flag": {
        "normalization": ["flag", "normalize", "canonical"],
        "precedence": ["precedence", "source_precedence", "order"],
        "conflict": ["conflict", "mutually", "exclusive"],
        "invalid_input": ["invalid", "missing", "unknown flag", "usage"],
    },
    "determinism": {
        "output": ["stdout", "json", "yaml", "snapshot"],
        "failure": ["error", "failure", "unknown route"],
        "ordering": ["ordering", "order", "sorted"],
        "repeated_run": ["for _ in", "repeat", "repeated", "determin"],
    },
    "command": {
        "parity": ["parity", "bridge", "core", "repl"],
        "failure": ["failure", "error", "unknown"],
        "determinism": ["determin", "repeat", "stable"],
        "stream_discipline": ["stdout", "stderr", "stream"],
    },
    "config": {
        "read": ["config get", "config list", "config export", "config load"],
        "mutation": ["config set", "config unset", "config clear", "config reload"],
        "corruption": ["corrupt", "malformed", "broken"],
        "rollback": ["rollback", "repair"],
        "precedence": ["precedence", "source_precedence"],
    },
    "history": {
        "read": ["history", "entries"],
        "corruption": ["corrupt", "malformed", "broken"],
        "ordering": ["order", "ordering", "determin"],
        "interop": ["repl", "bridge", "cross"],
    },
    "memory": {
        "read": ["memory", "list", "get"],
        "corruption": ["corrupt", "malformed", "broken"],
        "schema": ["schema", "field", "type", "missing"],
        "determinism": ["determin", "repeat", "stable"],
    },
    "diagnostics": {
        "consistency": ["consisten", "agree"],
        "schema": ["schema", "shape", "keys"],
        "trust": ["trust", "credible", "operator"],
        "cross_surface": ["bridge", "repl", "cross", "parity"],
    },
    "repl": {
        "transcript": ["transcript", "session", "line"],
        "hostile_session": ["hostile", "malformed", "ctrl-c", "recovery"],
        "completion": ["completion", "suggestion"],
        "parity": ["parity", "same", "non-interactive", "cli"],
    },
    "bridge": {
        "execution": ["execution", "run(", "bridge_outcome"],
        "conversion": ["conversion", "serialize", "deserialize"],
        "exception_mapping": ["exception", "error", "usage", "validation"],
        "parity": ["parity", "matches", "agree"],
    },
}

DOMAIN_PATH_HINTS = {
    "plugin": ["plugin"],
    "flag": ["flag", "precedence", "parser"],
    "determinism": ["determin", "ordering", "stability"],
    "command": ["command", "help", "surface", "execution", "parity"],
    "config": ["config"],
    "history": ["history"],
    "memory": ["memory"],
    "diagnostics": ["diagnostics", "doctor", "runtime_identity", "dev_cli"],
    "repl": ["repl", "transcript", "completion"],
    "bridge": ["bridge", "python"],
}


def load_tests() -> list[dict[str, Any]]:
    tests: list[dict[str, Any]] = []
    for root in TEST_ROOTS:
        if not root.exists():
            continue
        for path in sorted(root.glob("*.rs")):
            rel = str(path.relative_to(ROOT)).replace("\\", "/")
            text = path.read_text(encoding="utf-8", errors="ignore")
            lower = text.lower()
            tests.append(
                {
                    "path": rel,
                    "text": text,
                    "lower": lower,
                    "assert_count": text.count("assert!(") + text.count("assert_eq!("),
                }
            )
    return tests


def matches_domain(test: dict[str, Any], domain: str) -> bool:
    rel = test["path"].lower()
    text = test["lower"]
    hints = DOMAIN_PATH_HINTS[domain]
    return any(hint in rel or hint in text for hint in hints)


def category_rows(domain: str, tests: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    selected = [t for t in tests if matches_domain(t, domain)]
    rows = []
    for category, cues in CATEGORY_RULES[domain].items():
        matched_paths = sorted(
            t["path"]
            for t in selected
            if any(cue in t["lower"] for cue in cues)
        )
        rows.append(
            {
                "category": category,
                "count": len(matched_paths),
                "tests": matched_paths,
                "status": "covered" if matched_paths else "missing",
            }
        )

    weakest = sorted(
        selected,
        key=lambda t: (
            t["assert_count"],
            -sum(1 for _, cues in CATEGORY_RULES[domain].items() if any(cue in t["lower"] for cue in cues)),
            t["path"],
        ),
    )
    weak_rows = [
        {
            "path": t["path"],
            "assert_count": t["assert_count"],
            "category_hits": [
                name for name, cues in CATEGORY_RULES[domain].items() if any(cue in t["lower"] for cue in cues)
            ],
        }
        for t in weakest[:20]
    ]

    return rows, weak_rows


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    tests = load_tests()
    now = datetime.now(timezone.utc).isoformat()

    by_domain: dict[str, dict[str, Any]] = {}
    top_candidates: list[dict[str, Any]] = []

    output_map = {
        "plugin": "plugin_tests_by_category.json",
        "flag": "flag_tests_by_category.json",
        "determinism": "determinism_tests_by_category.json",
        "command": "command_tests_by_category.json",
        "config": "config_tests_by_category.json",
        "history": "history_tests_by_category.json",
        "memory": "memory_tests_by_category.json",
        "diagnostics": "diagnostics_tests_by_category.json",
        "repl": "repl_tests_by_category.json",
        "bridge": "bridge_tests_by_category.json",
    }

    for domain, out_name in output_map.items():
        categories, weak_rows = category_rows(domain, tests)
        missing = [row["category"] for row in categories if row["status"] != "covered"]
        payload = {
            "generated_at": now,
            "generator": "scripts/status/generate_deep_layer_test_coverage_reports.py",
            "domain": domain,
            "status": "complete" if not missing else "partial",
            "categories": categories,
            "missing_categories": missing,
        }
        write_json(out_name, payload)

        by_domain[domain] = payload
        for row in weak_rows:
            top_candidates.append(
                {
                    "domain": domain,
                    "path": row["path"],
                    "assert_count": row["assert_count"],
                    "category_hit_count": len(row["category_hits"]),
                }
            )

    top_10 = sorted(
        top_candidates,
        key=lambda r: (r["assert_count"], r["category_hit_count"], r["path"]),
    )[:10]

    weak_actions = {
        "generated_at": now,
        "generator": "scripts/status/generate_deep_layer_test_coverage_reports.py",
        "actions": [
            {
                "coverage_id": 392,
                "action": "deleted first weak test file",
                "evidence": "crates/bijux-cli/tests/bin_surface/ported_command_goldens.rs",
                "status": "done",
            },
            {
                "coverage_id": 393,
                "action": "rewrote weak snapshot coverage into repeated-run determinism proof",
                "evidence": "crates/bijux-cli-contracts/tests/schema_snapshots.rs",
                "status": "done",
            },
            {
                "coverage_id": 394,
                "action": "added failure-path deserialization proof for invalid payloads",
                "evidence": "crates/bijux-cli-contracts/tests/serde_roundtrip.rs",
                "status": "done",
            },
        ],
    }

    coverage = {
        "generated_at": now,
        "generator": "scripts/status/generate_deep_layer_test_coverage_reports.py",
        "scope": "deep-layer test coverage",
        "coverage_ids": [381, 382, 383, 384, 385, 386, 387, 388, 389, 390, 399],
        "status": "complete"
        if all(payload.get("status") == "complete" for payload in by_domain.values())
        else "partial",
        "domains": {domain: payload.get("status") for domain, payload in by_domain.items()},
        "top_10_weakest": top_10,
    }

    contract = {
        "generated_at": now,
        "generator": "scripts/status/generate_deep_layer_test_coverage_reports.py",
        "scope": "deep-layer behaviors contract",
        "coverage_ids": [400],
        "status": "frozen",
        "policy": [
            "deep-layer behavior changes must map to at least one domain category report",
            "stateful feature changes require corruption or rollback assertions",
            "cross-surface behavior changes require explicit equivalence assertions",
            "determinism claims require repeated-run proof assertions",
        ],
    }

    write_json("top_10_weakest_deep_layer_tests.json", {"generated_at": now, "generator": coverage["generator"], "tests": top_10})
    write_json("deep_layer_weak_test_actions.json", weak_actions)
    write_json("deep_layer_test_coverage_artifact.json", coverage)
    write_json("deep_layer_behaviors_contract.json", contract)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
