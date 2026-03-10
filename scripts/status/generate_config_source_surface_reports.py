#!/usr/bin/env python3
"""Generate config precedence/source parity + drift artifacts and frozen source-truth contract."""

from __future__ import annotations

import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
TEST_FILE = ROOT / "crates" / "bijux-cli-core" / "tests" / "bin_surface" / "config_source_precedence_matrix.rs"

REQUIRED_TESTS = {
    301: "cli_flags_override_env_backed_values_and_config_path",
    302: "env_overrides_file_and_file_overrides_default_with_missing_fallback",
    303: "env_overrides_file_and_file_overrides_default_with_missing_fallback",
    304: "cli_flags_override_env_backed_values_and_config_path",
    305: "env_overrides_file_and_file_overrides_default_with_missing_fallback",
    306: "malformed_and_duplicate_config_source_behavior_is_stable",
    307: "malformed_and_duplicate_config_source_behavior_is_stable",
    308: "source_metadata_and_dev_cli_env_precedence_are_reported",
    309: "source_metadata_and_dev_cli_env_precedence_are_reported",
    310: "source_metadata_and_dev_cli_env_precedence_are_reported",
    311: "source_reports_json_text_are_deterministic_ignore_noise_and_env_order",
    312: "source_reports_json_text_are_deterministic_ignore_noise_and_env_order",
    313: "source_reports_json_text_are_deterministic_ignore_noise_and_env_order",
    314: "source_reports_json_text_are_deterministic_ignore_noise_and_env_order",
    315: "source_reports_json_text_are_deterministic_ignore_noise_and_env_order",
    316: "cross_command_source_precedence_consistency",
    317: "cross_command_source_precedence_consistency",
    318: "cross_command_source_precedence_consistency",
}


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote artifacts/status/{name}")


def run_json(args: list[str], env: dict[str, str] | None = None) -> dict:
    proc = subprocess.run(args, check=True, capture_output=True, text=True, env=env)
    return json.loads(proc.stdout or "{}")


def main() -> int:
    source = TEST_FILE.read_text(encoding="utf-8")
    generated_at = datetime.now(timezone.utc).isoformat()

    todo_rows = [
        {
            "todo": todo,
            "test": fn_name,
            "status": "complete" if f"fn {fn_name}(" in source else "missing",
            "evidence": "crates/bijux-cli-core/tests/bin_surface/config_source_precedence_matrix.rs",
        }
        for todo, fn_name in sorted(REQUIRED_TESTS.items())
    ]

    temp_root = ROOT / "target" / "tmp" / "config-source-reports"
    temp_root.mkdir(parents=True, exist_ok=True)
    cfg = temp_root / "config.env"
    cfg.write_text("BIJUXCLI_ALPHA=from-file\n", encoding="utf-8")

    env = {**os.environ, "BIJUXCLI_CONFIG": str(cfg)}

    get_payload = run_json(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bijux-cli-core",
            "--",
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
        ],
        env=env,
    )
    dev_env_payload = run_json(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bijux-cli-core",
            "--",
            "dev",
            "cli",
            "env",
            "--format",
            "json",
            "--no-pretty",
        ],
        env=env,
    )

    source_path = get_payload.get("source_path")
    active_config = dev_env_payload.get("active", {}).get("config_file")
    precedence = dev_env_payload.get("source_precedence")

    drift_reasons: list[str] = []
    if source_path != active_config:
        drift_reasons.append("config_get.source_path does not match dev_cli_env.active.config_file")
    if precedence != ["flags", "env", "config", "defaults"]:
        drift_reasons.append("dev_cli_env.source_precedence does not match expected order")

    write_json(
        "config_source_parity_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_source_surface_reports.py",
            "scope": "todo 301-317 config precedence/source parity",
            "todo_rows": todo_rows,
            "comparison": {
                "config_get_source_path": source_path,
                "dev_cli_env_active_config_file": active_config,
                "dev_cli_env_source_precedence": precedence,
            },
            "status": "consistent" if not drift_reasons else "drift",
        },
    )

    write_json(
        "config_source_drift_artifact.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_source_surface_reports.py",
            "scope": "todo 318 config precedence/source drift",
            "drift_count": len(drift_reasons),
            "drift_reasons": drift_reasons,
            "status": "clean" if not drift_reasons else "drift",
        },
    )

    write_json(
        "config_source_precedence_contract.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_config_source_surface_reports.py",
            "domain": "config-source-precedence",
            "status": "frozen",
            "rule": "Config precedence truth must be observable, deterministic, and consistent across config get and dev cli env.",
            "evidence": [
                "crates/bijux-cli-core/tests/bin_surface/config_source_precedence_matrix.rs",
                "artifacts/status/config_source_parity_artifact.json",
                "artifacts/status/config_source_drift_artifact.json",
            ],
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
