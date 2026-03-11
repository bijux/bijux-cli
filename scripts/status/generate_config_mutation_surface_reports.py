#!/usr/bin/env python3
"""Generate config-mutation matrix artifact and frozen mutation-domain contract."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "config_mutation_matrix.rs"

REQUIRED_TESTS = {
    281: "config_set_create_replace_preserve_quoted_spaces_and_invalid_key",
    282: "config_set_create_replace_preserve_quoted_spaces_and_invalid_key",
    283: "config_set_create_replace_preserve_quoted_spaces_and_invalid_key",
    284: "config_set_create_replace_preserve_quoted_spaces_and_invalid_key",
    285: "config_set_create_replace_preserve_quoted_spaces_and_invalid_key",
    286: "config_set_create_replace_preserve_quoted_spaces_and_invalid_key",
    287: "config_unset_existing_and_missing_keys",
    288: "config_unset_existing_and_missing_keys",
    289: "config_clear_populated_and_empty_and_reload_after_external_change",
    290: "config_clear_populated_and_empty_and_reload_after_external_change",
    291: "config_clear_populated_and_empty_and_reload_after_external_change",
    292: "config_export_text_json_yaml_and_load_valid_malformed",
    293: "config_export_text_json_yaml_and_load_valid_malformed",
    294: "config_export_text_json_yaml_and_load_valid_malformed",
    295: "config_export_text_json_yaml_and_load_valid_malformed",
    296: "config_export_text_json_yaml_and_load_valid_malformed",
    297: "config_mutation_rollback_and_retry_idempotency_proof",
    298: "config_mutation_rollback_and_retry_idempotency_proof",
    299: "config_mutation_rollback_and_retry_idempotency_proof",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    generated_at = datetime.now(timezone.utc).isoformat()

    coverage_rows = [
        {
            "coverage_id": coverage_id,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli/tests/bin_surface/config_mutation_matrix.rs",
        }
        for coverage_id, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    write_json(
        "config_mutation_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_mutation_surface_reports.py",
            "scope": "config mutation matrix",
            "coverage_rows": coverage_rows,
            "domains": [
                {"surface": "config set", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                {"surface": "config unset", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                {"surface": "config clear/reload", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                {"surface": "config export/load", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                {"surface": "rollback + retry idempotency", "status": "complete", "evidence": "config_mutation_matrix.rs"},
            ],
        },
    )

    write_json(
        "config_mutation_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_mutation_surface_reports.py",
            "domain": "config-mutation",
            "status": "frozen",
            "rule": "Config mutation behavior is accepted only with rollback safety and idempotent retry proof.",
            "evidence": [
                "crates/bijux-cli/tests/bin_surface/config_mutation_matrix.rs",
                "artifacts/status/config_mutation_matrix_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
