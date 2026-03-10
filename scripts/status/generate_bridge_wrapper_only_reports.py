#!/usr/bin/env python3
"""Generate wrapper-only bridge closure evidence artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"
BRIDGE_TESTS = ROOT / "crates" / "bijux-cli-python" / "tests" / "bridge_bindings.rs"
CROSS_SURFACE_TESTS = ROOT / "crates" / "bijux-cli-bin" / "tests" / "cross_surface_equivalence.rs"


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


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    generated_at = stable_generated_at()
    bridge_duplicate = read_json(STATUS / "bridge_duplicate_law_report.json")
    duplicate_count = int(bridge_duplicate.get("summary", {}).get("duplicate_rule_count", 0))

    bridge_source = BRIDGE_TESTS.read_text(encoding="utf-8") if BRIDGE_TESTS.exists() else ""
    cross_surface_source = (
        CROSS_SURFACE_TESTS.read_text(encoding="utf-8") if CROSS_SURFACE_TESTS.exists() else ""
    )

    proof_tests = {
        "same_route_graph": (
            "binary_and_bridge_use_same_command_registry_contract",
            "route_registry_snapshots_match_across_binary_core_and_bridge",
        ),
        "same_command_registry": ("binary_and_bridge_use_same_command_registry_contract",),
        "same_output_envelope": ("binary_and_bridge_use_same_output_envelope_shape",),
        "same_exit_mappings": ("binary_and_bridge_use_same_exit_mapping_for_unknown_route",),
        "same_namespace_law": ("binary_and_bridge_use_same_namespace_rejection_logic",),
        "same_config_precedence": ("execution_path_keeps_config_precedence_identical_between_binary_and_bridge",),
    }

    test_presence: dict[str, dict[str, Any]] = {}
    for key, names in proof_tests.items():
        present = []
        for name in names:
            in_bridge = f"fn {name}(" in bridge_source
            in_cross = f"fn {name}(" in cross_surface_source
            if in_bridge or in_cross:
                present.append(name)
        test_presence[key] = {"required": list(names), "present": present, "ok": len(present) == len(names)}

    all_proofs_ok = all(item["ok"] for item in test_presence.values())
    wrapper_only_status = duplicate_count == 0 and all_proofs_ok

    payload = {
        "generated_at": generated_at,
        "generator": "scripts/status/generate_bridge_wrapper_only_reports.py",
        "scope": "bridge wrapper-only closure",
        "sources": [
            "artifacts/status/bridge_duplicate_law_report.json",
            "crates/bijux-cli-python/tests/bridge_bindings.rs",
            "crates/bijux-cli-bin/tests/cross_surface_equivalence.rs",
        ],
        "duplicate_law": {
            "duplicate_rule_count": duplicate_count,
            "status": "clean" if duplicate_count == 0 else "duplicates-found",
        },
        "proof_tests": test_presence,
        "bridge_vs_binary_parity_artifact": "artifacts/parity/binary_vs_python_bridge_parity_report.json",
        "ci_gates": {
            "binary_bridge_parity_gate": "scripts/parity/check_binary_bridge_parity_gate.py --enforce",
            "bridge_duplicate_law_policy_gate": "scripts/status/enforce_bridge_duplicate_law_policy.py",
        },
        "status": "green" if wrapper_only_status else "open",
        "wrapper_only_frozen": wrapper_only_status,
    }
    write_json(STATUS / "bridge_wrapper_only_closure_report.json", payload)

    text = [
        "Bridge Wrapper-Only Closure Report",
        f"status: {payload['status']}",
        f"wrapper-only frozen: {payload['wrapper_only_frozen']}",
        f"duplicate rule count: {duplicate_count}",
        "",
        "proof tests:",
    ]
    for key, item in test_presence.items():
        text.append(f"- {key}: {item['ok']}")
    text.append("")
    text.append("ci gates:")
    text.append(f"- {payload['ci_gates']['binary_bridge_parity_gate']}")
    text.append(f"- {payload['ci_gates']['bridge_duplicate_law_policy_gate']}")
    (STATUS / "bridge_wrapper_only_closure_report.txt").write_text(
        "\n".join(text) + "\n", encoding="utf-8"
    )
    print("wrote artifacts/status/bridge_wrapper_only_closure_report.json")
    print("wrote artifacts/status/bridge_wrapper_only_closure_report.txt")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
