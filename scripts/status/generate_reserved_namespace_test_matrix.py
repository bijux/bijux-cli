#!/usr/bin/env python3
"""Generate reserved-namespace law test matrix artifact."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_namespace_law.rs"


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


def main() -> None:
    text = TEST_FILE.read_text(encoding="utf-8") if TEST_FILE.exists() else ""
    test_names = set(re.findall(r"fn\s+([a-z0-9_]+)\s*\(", text))

    rows = [
        (1, "rejects_plugin_namespace_cli"),
        (2, "rejects_plugin_namespace_dev"),
        (3, "rejects_plugin_namespace_help"),
        (4, "rejects_plugin_namespace_version"),
        (5, "rejects_plugin_namespace_doctor"),
        (6, "rejects_plugin_namespace_plugins"),
        (7, "rejects_plugin_namespace_repl"),
        (8, "rejects_official_product_namespace_dag"),
        (9, "rejects_official_product_namespace_atlas"),
        (10, "rejects_normalized_collision_my_plugin_vs_my_plugin_hyphen"),
        (11, "rejects_case_insensitive_normalized_collision"),
        (12, "rejects_namespace_with_leading_digit"),
        (13, "rejects_namespace_with_whitespace"),
        (14, "rejects_namespace_with_shell_hostile_punctuation"),
        (15, "rejects_empty_namespace"),
        (16, "rejects_namespace_differing_only_by_hidden_alias_collision"),
        (17, "rejection_messages_explain_the_reason_clearly"),
        (18, "json_error_envelopes_for_namespace_rejection_are_stable"),
        (19, "text_errors_for_namespace_rejection_are_stable"),
    ]

    payload = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_reserved_namespace_test_matrix.py",
        "scope": "todo 1-20 plugin namespace law tests",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/plugin_namespace_law.rs",
            }
            for todo, name in rows
        ],
    }
    payload["summary"] = {
        "complete": sum(1 for row in payload["rows"] if row["status"] == "complete"),
        "missing": sum(1 for row in payload["rows"] if row["status"] == "missing"),
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "reserved_namespace_test_matrix.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/reserved_namespace_test_matrix.json")


if __name__ == "__main__":
    main()
