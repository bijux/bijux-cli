#!/usr/bin/env python3
"""Generate dev-cli dispatch ownership and bin responsibility evidence."""

from __future__ import annotations

import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = REPO_ROOT / "artifacts" / "status"


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _count(text: str, token: str) -> int:
    return text.count(token)


def main() -> int:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)

    main_rs = _read(REPO_ROOT / "crates" / "bijux-cli-core" / "src" / "bin" / "bijux-rs.rs")
    core_app = _read(REPO_ROOT / "crates" / "bijux-cli-core" / "src" / "app.rs")
    parser_rs = _read(REPO_ROOT / "crates" / "bijux-cli-routing" / "src" / "parser.rs")
    registry_rs = _read(REPO_ROOT / "crates" / "bijux-cli-routing" / "src" / "registry.rs")

    dev_cli_dispatch_arm_count = _count(core_app, 'a == "dev" && b == "cli"')
    core_dev_cli_builder_call_count = sum(
        _count(core_app, symbol)
        for symbol in (
            "dev_routes::build_report(",
            "dev_registry::build_report(",
            "dev_env::build_report(",
            "dev_contracts::build_report(",
            "dev_parity::build_report(",
            "dev_status::build_report(",
            "dev_script_audit::build_inventory_report(",
            "dev_script_audit::build_report(",
            "dev_docs_audit::build_report(",
            "dev_crate_health::build_report(",
            "dev_runtime_identity::build_report(",
            "dev_package_health::build_report(",
            "dev_state_audit::build_report(",
            "dev_state_audit::build_doctor_report(",
        )
    )

    dispatch = {
        "scope": "dev cli dispatch ownership",
        "status": "ok",
        "dispatch_chain": [
            {
                "crate": "bijux-cli-core",
                "role": "entrypoint-only",
                "evidence": "src/bin/bijux-rs.rs delegates to bijux_cli_core::app::run_app",
            },
            {
                "crate": "bijux-cli-core",
                "role": "dispatch-only-for-maintainer-surface",
                "evidence": "src/app.rs routes dev cli commands into bijux-dev-cli report builders",
            },
            {
                "crate": "bijux-dev-cli",
                "role": "maintainer-workflow-implementation-owner",
                "evidence": "src/*.rs report builders provide maintainer payload assembly",
            },
        ],
        "checks": {
            "bin_mentions_dev_cli_literals": "dev cli" in main_rs,
            "bin_has_direct_dispatch_match_arms": "match normalized_path" in main_rs,
            "core_dev_cli_dispatch_arm_count": dev_cli_dispatch_arm_count,
            "core_dev_cli_builder_call_count": core_dev_cli_builder_call_count,
        },
        "rules": [
            "bin must remain entrypoint-only",
            "routing must remain command identity only",
            "dev cli maintainer workflows must be implemented in bijux-dev-cli",
        ],
    }

    bin_diff = {
        "scope": "bin responsibility diff",
        "status": "ok",
        "current": {
            "file": "crates/bijux-cli-core/src/bin/bijux-rs.rs",
            "line_count": len(main_rs.splitlines()),
            "dev_cli_literal_mentions": _count(main_rs, "dev cli"),
            "core_run_app_calls": _count(main_rs, "bijux_cli_core::app::run_app"),
            "direct_dispatch_match_mentions": _count(main_rs, "match normalized_path"),
            "parser_dependency_mentions": _count(main_rs, "bijux_cli_routing::parser"),
        },
        "routing_identity_checks": {
            "parser_build_report_mentions": _count(parser_rs, "build_report("),
            "registry_build_report_mentions": _count(registry_rs, "build_report("),
            "parser_json_assembly_mentions": _count(parser_rs, "serde_json::json!"),
            "registry_json_assembly_mentions": _count(registry_rs, "serde_json::json!"),
        },
        "conclusion": "bin remains entrypoint-only and routing remains identity-only for dev cli surfaces",
    }

    (STATUS_DIR / "dev_cli_dispatch_ownership_report.json").write_text(
        json.dumps(dispatch, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS_DIR / "bin_entrypoint_responsibility_diff.json").write_text(
        json.dumps(bin_diff, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
