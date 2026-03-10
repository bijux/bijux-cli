#!/usr/bin/env python3
"""Generate parser abuse hardening report artifact."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "parser_abuse_report.json"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "routing" / "parser_abuse.rs"


def main() -> int:
    text = TEST_FILE.read_text(encoding="utf-8")
    checks = {
        "401": "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
        "402": "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
        "403": "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
        "404": "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
        "405": "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
        "406": "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
        "407": "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
        "408": "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
        "409": "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
        "410": "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
        "411": "parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths",
        "412": "parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths",
        "413": "unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity",
        "414": "unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity",
        "415": "plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots",
        "416": "plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots",
        "417": "route_tree_and_command_tree_are_deterministic_under_shuffled_plugin_registration",
        "418": "command_tree_export_is_stable_across_repeated_calls",
    }

    rows = []
    for todo, test_name in checks.items():
        rows.append(
            {
                "todo": int(todo),
                "status": "complete" if test_name in text else "missing",
                "evidence_test": f"crates/bijux-cli/tests/routing/parser_abuse.rs::{test_name}",
            }
        )

    payload = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_parser_abuse_report.py",
        "scope": "401-420 parser and routing hardening wave",
        "rows": rows,
        "summary": {
            "complete": sum(1 for row in rows if row["status"] == "complete"),
            "missing": sum(1 for row in rows if row["status"] == "missing"),
        },
        "required_before_major_release_claims": True,
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
