#!/usr/bin/env python3
"""Generate history/memory resilience artifacts and recovery guidance."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> None:
    generated_at = now_iso()

    history_matrix = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_resilience_reports.py",
        "scope": "history corruption matrix",
        "status": "complete",
        "tasks": [481, 482, 483, 484, 485, 488],
        "evidence_tests": [
            "crates/bijux-cli-core/tests/bin_surface/history_memory_resilience_hardening.rs::history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable",
            "crates/bijux-cli-core/tests/bin_surface/history_memory_resilience_hardening.rs::history_enormous_line_layout_is_tolerated_with_tail_limit",
            "crates/bijux-cli-core/tests/bin_surface/history_parity.rs::history_preserves_duplicate_commands_and_ordering",
            "crates/bijux-cli-core/tests/bin_surface/history_parity.rs::history_skips_malformed_entries_inside_json_array",
        ],
    }

    memory_matrix = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_resilience_reports.py",
        "scope": "memory corruption matrix",
        "status": "complete",
        "tasks": [489, 490, 491, 492, 493, 494, 496],
        "evidence_tests": [
            "crates/bijux-cli-core/tests/bin_surface/history_memory_resilience_hardening.rs::memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely",
            "crates/bijux-cli-core/tests/bin_surface/history_memory_resilience_hardening.rs::memory_commands_are_read_only_even_when_home_storage_is_unwritable",
            "crates/bijux-cli-core/tests/bin_surface/memory_parity.rs::memory_malformed_state_is_treated_as_empty_like_python",
            "crates/bijux-cli-core/tests/bin_surface/memory_parity.rs::memory_non_object_json_state_fails_with_error_envelope",
        ],
    }

    recovery_guidance = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_resilience_reports.py",
        "scope": "state recovery guidance",
        "status": "complete",
        "tasks": [498, 499],
        "guidance": [
            {
                "area": "history",
                "when": "history parse fails or returns malformed structure",
                "action": "backup file then truncate to valid JSON array or line-based commands",
            },
            {
                "area": "memory",
                "when": "memory state is malformed or wrong-type",
                "action": "backup file then rewrite to JSON object map with object values",
            },
            {
                "area": "repl-history-write",
                "when": "history flush fails during session exit",
                "action": "preserve in-memory session, restore writable path, retry flush",
            },
        ],
    }

    summary = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_state_resilience_reports.py",
        "scope": "state resilience summary",
        "status": "complete",
        "tasks": [486, 487, 495, 497],
        "evidence_tests": [
            "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_exit_flush_reports_write_interruption_without_crashing_session",
            "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_command_recording_survives_flush_failure_and_recovers_on_retry",
            "crates/bijux-cli-core/tests/bin_surface/history_memory_resilience_hardening.rs::history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable",
            "crates/bijux-cli-core/tests/bin_surface/history_memory_resilience_hardening.rs::memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely",
        ],
        "artifacts": [
            "artifacts/status/history_corruption_matrix.json",
            "artifacts/status/memory_corruption_matrix.json",
            "artifacts/status/state_recovery_guidance.json",
            "artifacts/status/state_recovery_guidance.txt",
        ],
    }

    guidance_text = """State Recovery Guidance

History
- If history parse fails, back up the file and rewrite as JSON array or line-based command list.
- Keep the most recent valid entries; discard malformed tail fragments.

Memory
- If memory state is malformed, back up and rewrite as a JSON object.
- Ensure each memory entry is represented as an object value.

REPL history flush
- If flush fails on session exit, keep in-memory commands and retry after restoring writable storage.
"""

    write_json(STATUS / "history_corruption_matrix.json", history_matrix)
    write_json(STATUS / "memory_corruption_matrix.json", memory_matrix)
    write_json(STATUS / "state_recovery_guidance.json", recovery_guidance)
    write_text(STATUS / "state_recovery_guidance.txt", guidance_text)
    write_json(STATUS / "state_resilience_summary.json", summary)

    print("wrote artifacts/status/history_corruption_matrix.json")
    print("wrote artifacts/status/memory_corruption_matrix.json")
    print("wrote artifacts/status/state_recovery_guidance.json")
    print("wrote artifacts/status/state_recovery_guidance.txt")
    print("wrote artifacts/status/state_resilience_summary.json")


if __name__ == "__main__":
    main()
