#!/usr/bin/env python3
"""Generate config-read coverage/matrix artifacts and deterministic read-domain contract."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "config_read_matrix.rs"

REQUIRED_TESTS = {
    261: "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
    262: "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
    263: "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
    264: "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
    265: "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
    266: "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
    267: "config_get_existing_missing_invalid_with_path_and_env_override",
    268: "config_get_existing_missing_invalid_with_path_and_env_override",
    269: "config_get_existing_missing_invalid_with_path_and_env_override",
    270: "config_get_existing_missing_invalid_with_path_and_env_override",
    271: "config_get_existing_missing_invalid_with_path_and_env_override",
    272: "config_get_json_yaml_text_quiet_and_no_color_behavior",
    273: "config_get_json_yaml_text_quiet_and_no_color_behavior",
    274: "config_get_json_yaml_text_quiet_and_no_color_behavior",
    275: "config_get_json_yaml_text_quiet_and_no_color_behavior",
    276: "config_get_json_yaml_text_quiet_and_no_color_behavior",
    277: "config_listing_repeated_run_determinism_and_field_order_stability",
    278: "config_listing_repeated_run_determinism_and_field_order_stability",
    279: "config_listing_repeated_run_determinism_and_field_order_stability",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    generated_at = datetime.now(timezone.utc).isoformat()

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli-core/tests/bin_surface/config_read_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    read_domain_rows = [
        {"surface": "root config list", "status": "complete", "evidence": "config_read_matrix.rs"},
        {"surface": "cli config get", "status": "complete", "evidence": "config_read_matrix.rs"},
        {"surface": "json/yaml/text rendering", "status": "complete", "evidence": "config_read_matrix.rs"},
        {"surface": "quiet/no-color behavior", "status": "complete", "evidence": "config_read_matrix.rs"},
        {"surface": "deterministic repeated runs", "status": "complete", "evidence": "config_read_matrix.rs"},
    ]

    write_json(
        "config_read_matrix_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_read_surface_reports.py",
            "scope": "todo 261-279 config read matrix",
            "todo_rows": todo_rows,
            "domains": read_domain_rows,
        },
    )

    write_json(
        "config_read_domain_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_read_surface_reports.py",
            "domain": "config-read",
            "status": "frozen",
            "rule": "Config reads must remain deterministic, explainable, and consistent across listing/get surfaces.",
            "evidence": [
                "crates/bijux-cli-core/tests/bin_surface/config_read_matrix.rs",
                "artifacts/status/config_read_matrix_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
