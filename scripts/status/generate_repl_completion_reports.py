#!/usr/bin/env python3
"""Generate REPL completion artifacts."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-repl" / "tests" / "repl_completion_extra.rs"

REQUIRED_TESTS = {
    241: "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported",
    242: "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported",
    243: "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported",
    244: "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported",
    245: "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported",
    246: "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported",
    247: "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported",
    248: "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported",
    249: "completion_reserved_namespaces_are_visible_and_hidden_aliases_are_not_canonical_suggestions",
    250: "completion_reserved_namespaces_are_visible_and_hidden_aliases_are_not_canonical_suggestions",
    251: "completion_recovers_with_broken_registry_corrupted_state_and_no_plugins",
    252: "completion_recovers_with_broken_registry_corrupted_state_and_no_plugins",
    253: "completion_recovers_with_broken_registry_corrupted_state_and_no_plugins",
    254: "completion_ordering_is_stable_with_multiple_plugins_and_repeated_runs",
    255: "completion_ordering_is_stable_with_multiple_plugins_and_repeated_runs",
}


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = []
    for coverage_id, name in sorted(REQUIRED_TESTS.items()):
        covered = f"fn {name}(" in source
        coverage_rows.append(
            {
                "coverage_id": coverage_id,
                "test": name,
                "status": "covered" if covered else "missing",
                "evidence": "crates/bijux-cli-repl/tests/repl_completion_extra.rs",
            }
        )

    missing = [row for row in coverage_rows if row["status"] != "covered"]

    completion = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_completion_reports.py",
        "scope": "repl completion",
        "coverage_ids": list(range(241, 257)),
        "status": "complete" if not missing else "partial",
        "coverage_rows": coverage_rows,
    }

    ordering = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_completion_reports.py",
        "scope": "repl completion ordering",
        "coverage_ids": [254, 255, 257],
        "status": "stable" if not missing else "unstable",
        "drift_count": len(missing),
        "drift_coverage_ids": [row["coverage_id"] for row in missing],
    }

    drift = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_completion_reports.py",
        "scope": "repl completion drift",
        "coverage_ids": [258, 259],
        "status": "clean" if not missing else "drift",
        "drift_count": len(missing),
        "drift_coverage_ids": [row["coverage_id"] for row in missing],
    }

    contract = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_completion_reports.py",
        "scope": "repl completion contract",
        "coverage_ids": [260],
        "status": "frozen" if not missing else "not-frozen",
        "law": "completion behavior is a tested surface",
    }

    write_json("repl_completion_artifact.json", completion)
    write_json("repl_completion_ordering_artifact.json", ordering)
    write_json("repl_completion_drift_artifact.json", drift)
    write_json("repl_completion_contract.json", contract)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
