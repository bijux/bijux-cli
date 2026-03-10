#!/usr/bin/env python3
"""Generate namespace abuse and reservation inventory artifacts."""

from __future__ import annotations

import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
ROUTING_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "routing" / "registry_namespace_policy.rs"
PLUGIN_TEST = ROOT / "crates" / "bijux-cli-plugin" / "tests" / "plugin_namespace_regression.rs"
CLI_TEST = ROOT / "crates" / "bijux-cli" / "tests" / "bin_surface" / "plugin_cli_lifecycle.rs"
CONSTANTS = ROOT / "crates" / "bijux-cli-plugin" / "src" / "constants.rs"
PRODUCT_REGISTRY = ROOT / "docs" / "constitution" / "official_product_namespace_registry.json"


def extract_array(text: str, const_name: str) -> list[str]:
    marker = f"pub const {const_name}: &[&str] ="
    idx = text.find(marker)
    if idx < 0:
        return []
    chunk = text[idx:]
    start = chunk.find('[')
    end = chunk.find("];")
    if start < 0 or end < 0:
        return []
    return re.findall(r'"([^"]+)"', chunk[start:end])


def main() -> int:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        generated_at = datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    else:
        generated_at = "1970-01-01T00:00:00+00:00"
    routing_text = ROUTING_TEST.read_text(encoding="utf-8")
    plugin_text = PLUGIN_TEST.read_text(encoding="utf-8")
    cli_text = CLI_TEST.read_text(encoding="utf-8")
    constants = CONSTANTS.read_text(encoding="utf-8")
    product_registry = json.loads(PRODUCT_REGISTRY.read_text(encoding="utf-8"))

    abuse_checks = [
        ("421", "official_reserved_namespaces_take_precedence"),
        ("422", "official_reserved_namespaces_take_precedence"),
        ("423", "official_reserved_namespaces_take_precedence"),
        ("424", "official_reserved_namespaces_take_precedence"),
        ("425", "official_reserved_namespaces_take_precedence"),
        ("426", "rejects_future_official_product_namespaces"),
        ("427", "normalized_and_case_folded_namespace_collisions_are_rejected"),
        ("428", "normalized_and_case_folded_namespace_collisions_are_rejected"),
        ("429", "hidden_alias_paths_remain_builtin_when_namespace_resembles_alias_tail"),
        ("430", "plugin_uninstall_followed_by_reinstall_succeeds"),
        ("431", "concurrent_install_attempts_same_namespace_keep_registry_consistent"),
        ("432", "concurrent_registration_on_normalized_equivalent_namespaces_yields_single_winner"),
        ("433", "namespace_conflict_failure_does_not_mutate_existing_registry_entries"),
        ("434", "reserved_namespace_rejections_emit_clear_machine_readable_errors"),
        ("435", "reserved_namespace_rejections_emit_clear_machine_readable_errors"),
        ("436", "reserved_names_and_explain_outputs_are_stable_for_rejected_namespaces"),
        ("437", "reserved_names_and_explain_outputs_are_stable_for_rejected_namespaces"),
    ]

    evidence_text = "\n".join([routing_text, plugin_text, cli_text])
    rows = []
    for todo, test_name in abuse_checks:
        rows.append(
            {
                "todo": int(todo),
                "status": "complete" if test_name in evidence_text else "missing",
                "evidence_test": test_name,
            }
        )

    abuse_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_namespace_reservation_reports.py",
        "scope": "421-440 namespace and reservation abuse hardening",
        "rows": rows,
        "summary": {
            "complete": sum(1 for row in rows if row["status"] == "complete"),
            "missing": sum(1 for row in rows if row["status"] == "missing"),
        },
    }

    reserved = sorted(set(extract_array(constants, "RESERVED_NAMESPACES")))
    core = sorted(set(extract_array(constants, "CORE_NAMESPACES")))
    future = sorted(set(extract_array(constants, "FUTURE_PRODUCT_NAMESPACES")))
    inventory_payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_namespace_reservation_reports.py",
        "reserved_namespaces": reserved,
        "core_namespaces": core,
        "future_product_namespaces": future,
        "canonical_source": "docs/constitution/official_product_namespace_registry.json",
        "registry_entries": product_registry.get("entries", []),
    }

    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / "namespace_abuse_report.json").write_text(
        json.dumps(abuse_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (STATUS / "reserved_namespace_inventory.json").write_text(
        json.dumps(inventory_payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print("wrote artifacts/status/namespace_abuse_report.json")
    print("wrote artifacts/status/reserved_namespace_inventory.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
