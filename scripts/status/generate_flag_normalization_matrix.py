#!/usr/bin/env python3
"""Generate flag normalization matrix artifact for TODOs 81-100."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "flag_normalization_matrix.rs"


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
        (81, "global_flags_before_namespace_are_accepted"),
        (82, "global_flags_after_namespace_are_accepted_when_supported"),
        (83, "global_flags_before_and_after_namespace_normalize_to_same_intent"),
        (84, "repeated_format_flags_are_rejected_deterministically"),
        (85, "repeated_pretty_flags_are_rejected_deterministically"),
        (86, "repeated_no_pretty_flags_are_rejected_deterministically"),
        (87, "repeated_quiet_flags_are_rejected_deterministically"),
        (88, "repeated_trace_flags_are_rejected_deterministically"),
        (89, "repeated_color_flags_are_rejected_deterministically"),
        (90, "repeated_config_flags_are_rejected_deterministically"),
        (91, "conflicting_pretty_and_no_pretty_have_stable_resolution"),
        (92, "conflicting_color_always_and_never_are_rejected"),
        (93, "invalid_format_value_is_rejected"),
        (94, "invalid_color_value_is_rejected"),
        (95, "missing_value_after_config_flag_is_rejected"),
        (96, "missing_value_after_format_flag_is_rejected"),
        (97, "unknown_global_flag_at_root_is_rejected"),
        (98, "unknown_local_flag_in_grouped_command_is_rejected"),
        (99, "mixed_global_local_flag_ordering_abuse_is_rejected"),
    ]

    payload = {
        "generated_at": stable_generated_at(),
        "generator": "scripts/status/generate_flag_normalization_matrix.py",
        "scope": "todo 81-100 flag normalization tests",
        "rows": [
            {
                "todo": todo,
                "test_name": name,
                "status": "complete" if name in test_names else "missing",
                "evidence": "crates/bijux-cli/tests/bin_surface/flag_normalization_matrix.rs",
            }
            for todo, name in rows
        ],
    }
    payload["summary"] = {
        "complete": sum(1 for row in payload["rows"] if row["status"] == "complete"),
        "missing": sum(1 for row in payload["rows"] if row["status"] == "missing"),
        "artifact_todo": 100,
        "artifact_path": "artifacts/status/flag_normalization_matrix.json",
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "flag_normalization_matrix.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/flag_normalization_matrix.json")


if __name__ == "__main__":
    main()
