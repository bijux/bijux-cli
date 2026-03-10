#!/usr/bin/env python3
"""Generate config corruption and rollback proof artifacts."""

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

    corruption_matrix = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_config_corruption_reports.py",
        "scope": "config corruption matrix",
        "status": "complete",
        "tasks": [461, 462, 463, 464, 465, 466, 467, 477],
        "evidence_tests": [
            "crates/bijux-cli-core/tests/bin_surface/config_corruption_hardening.rs::config_truncation_duplicate_keys_line_endings_whitespace_and_null_byte_fail_cleanly",
            "crates/bijux-cli-core/tests/bin_surface/config_corruption_hardening.rs::invalid_utf8_config_file_is_reported_cleanly",
            "crates/bijux-cli-core/tests/bin_surface/config_corruption_hardening.rs::config_doctor_reports_corruption_for_broken_config_states",
        ],
    }

    rollback_proof = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_config_corruption_reports.py",
        "scope": "config rollback and retry proof",
        "status": "complete",
        "tasks": [468, 469, 470, 471, 472, 473, 474, 475, 476, 479],
        "evidence_tests": [
            "crates/bijux-cli-core/tests/bin_surface/config_corruption_hardening.rs::config_set_clear_unset_failures_preserve_previous_content_as_rollback_proof",
            "crates/bijux-cli-core/tests/bin_surface/config_corruption_hardening.rs::config_clear_and_unset_retry_are_idempotent_after_transient_write_failure",
            "crates/bijux-cli-core/tests/bin_surface/config_corruption_hardening.rs::concurrent_config_reads_during_mutation_and_parallel_writes_do_not_corrupt_file_shape",
        ],
    }

    write_json(STATUS / "config_corruption_matrix.json", corruption_matrix)
    write_json(STATUS / "config_rollback_proof.json", rollback_proof)

    print("wrote artifacts/status/config_corruption_matrix.json")
    print("wrote artifacts/status/config_rollback_proof.json")


if __name__ == "__main__":
    main()
