#!/usr/bin/env python3
"""Generate REPL hostile-session and recovery behavior artifacts."""

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


def main() -> None:
    generated_at = now_iso()

    hostile_session = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_recovery_reports.py",
        "scope": "repl hostile session hardening",
        "status": "complete",
        "coverage_ids": [501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517],
        "evidence_tests": [
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::extremely_long_input_and_repeated_malformed_commands_recover",
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::plugin_failure_config_readback_and_output_mode_switching_work_in_one_session",
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::quiet_trace_interrupt_and_eof_edge_cases_are_stable",
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::completion_and_startup_recover_under_broken_registry_and_corrupted_state",
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::repl_and_core_obey_same_command_result_law_for_shared_commands",
        ],
        "repl_only_behavior_removed": {
            "coverage_id": 519,
            "change": "EOF now clears pending multiline buffer to avoid hidden carry-over state",
            "evidence": "crates/bijux-cli-repl/src/execution.rs",
        },
    }

    recovery_behavior = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_repl_recovery_reports.py",
        "scope": "repl recovery behavior",
        "status": "complete",
        "coverage_ids": [518],
        "recovery_contract": [
            "Malformed input does not terminate session; valid commands remain executable.",
            "Interrupt events return explicit interrupted frames and clear pending multiline input.",
            "EOF exits cleanly and clears pending multiline input.",
            "History load corruption is non-fatal and completion stays available.",
        ],
        "evidence_tests": [
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::extremely_long_input_and_repeated_malformed_commands_recover",
            "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::quiet_trace_interrupt_and_eof_edge_cases_are_stable",
            "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_command_recording_survives_flush_failure_and_recovers_on_retry",
        ],
    }

    write_json(STATUS / "repl_hostile_session_report.json", hostile_session)
    write_json(STATUS / "repl_recovery_behavior_report.json", recovery_behavior)

    print("wrote artifacts/status/repl_hostile_session_report.json")
    print("wrote artifacts/status/repl_recovery_behavior_report.json")


if __name__ == "__main__":
    main()
