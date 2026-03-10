#!/usr/bin/env python3
"""Validate runtime identity and bridge-routing law invariants."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "runtime_unity_report.json"


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (FileNotFoundError, UnicodeDecodeError):
        return ""


def check_python_bridge_uses_core_entrypoint() -> tuple[bool, str]:
    bindings = read_text(ROOT / "crates" / "bijux-cli-python" / "src" / "bindings.rs")
    ok = "use bijux_cli_core::app::{run_app, AppRunResult};" in bindings and "match run_app(argv)" in bindings
    detail = "python bridge executes through bijux_cli_core::app::run_app"
    if not ok:
        detail = "python bridge does not use bijux_cli_core::app::run_app as canonical entrypoint"
    return ok, detail


def check_python_bridge_has_no_separate_routing() -> tuple[bool, str]:
    txt = read_text(ROOT / "crates" / "bijux-cli-python" / "src" / "bindings.rs")
    forbidden = ["parse_intent", "RouteRegistry", "root_command(", "render_command_help("]
    hits = [token for token in forbidden if token in txt]
    ok = not hits
    return ok, "no separate routing rules in python bridge" if ok else f"forbidden routing symbols found: {', '.join(hits)}"


def check_python_bridge_has_no_separate_exit_mapping() -> tuple[bool, str]:
    txt = read_text(ROOT / "crates" / "bijux-cli-python" / "src" / "bindings.rs")
    ok = "result.exit_code" in txt and "classify_failure(result.exit_code" in txt
    if not ok:
        return False, "python bridge does not derive exit mapping from core result"
    return True, "python bridge derives exit mapping from core result"


def check_python_bridge_has_no_separate_output_semantics() -> tuple[bool, str]:
    txt = read_text(ROOT / "crates" / "bijux-cli-python" / "src" / "bindings.rs")
    ok = "select_primary_stream(&result)" in txt and "result.stdout" in txt and "result.stderr" in txt
    if not ok:
        return False, "python bridge output envelope is not derived from core streams"
    return True, "python bridge output envelope derived from core streams"


def check_bin_crate_logic_boundary() -> tuple[bool, str]:
    main_rs = read_text(ROOT / "crates" / "bijux-cli-bin" / "src" / "main.rs")
    core_rs = read_text(ROOT / "crates" / "bijux-cli-core" / "src" / "app.rs")
    ok = (
        "bijux_cli_core::app::run_app" in main_rs
        and "parse_intent" not in main_rs
        and "route_response" not in main_rs
        and "run_app(" in core_rs
    )
    detail = "bin delegates behavior to core::app::run_app"
    if not ok:
        detail = "bin/core boundary check failed: bin may own behavior beyond IO + process wiring"
    return ok, detail


def check_scripts_do_not_bypass_supported_rust_entrypoint() -> tuple[bool, str]:
    scripts_dir = ROOT / "scripts"
    bypass_hits: list[str] = []
    allowed = {
        "scripts/capture_python_behavior.py",
        "scripts/parity/run_rust_python_parity.py",
        "scripts/status/check_runtime_unity.py",
    }
    if scripts_dir.exists():
        for path in scripts_dir.rglob("*"):
            if not path.is_file():
                continue
            rel = str(path.relative_to(ROOT))
            if rel in allowed:
                continue
            txt = read_text(path)
            if "./bin/bijux" in txt or "python -m bijux_cli" in txt:
                bypass_hits.append(rel)
    ok = not bypass_hits
    if ok:
        return True, "no unsupported script bypasses for Rust-supported commands"
    return False, f"script bypass candidates: {', '.join(sorted(bypass_hits))}"


def check_package_metadata_points_to_bijux() -> tuple[bool, str]:
    pyproject = read_text(ROOT / "pyproject.toml")
    has_bijux = "[project.scripts]" in pyproject and "bijux = " in pyproject
    forbidden = [name for name in ("bijux-rs", "bijux-cli-rs", "bijux-cli-py") if f"{name} = " in pyproject]
    ok = has_bijux and not forbidden
    if ok:
        return True, "package metadata points users to bijux"
    return False, f"invalid entrypoint metadata: missing bijux or forbidden aliases {forbidden}"


def check_runtime_identity_command_visible() -> tuple[bool, str]:
    parser = read_text(ROOT / "crates" / "bijux-cli-routing" / "src" / "parser.rs")
    registry = read_text(ROOT / "crates" / "bijux-cli-routing" / "src" / "registry.rs")
    core = read_text(ROOT / "crates" / "bijux-cli-core" / "src" / "app.rs")
    ok = all(
        token in parser + registry + core
        for token in (
            "runtime-identity",
            "dev cli runtime-identity",
            "canonical_user_binary",
        )
    )
    return ok, "dev cli runtime-identity command is available" if ok else "runtime identity command route is incomplete"


def check_one_law_document_exists() -> tuple[bool, str]:
    path = ROOT / "docs" / "architecture" / "runtime_identity_law.md"
    txt = read_text(path)
    ok = path.exists() and "one law, many entrypoints" in txt.lower()
    return ok, "runtime law document exists" if ok else "missing docs/architecture/runtime_identity_law.md"


def load_parity_assertions() -> tuple[bool, str]:
    report = ROOT / "artifacts" / "status" / "current_rust_state.json"
    if not report.exists():
        return False, "missing artifacts/status/current_rust_state.json"
    data = json.loads(read_text(report))
    assertions = data.get("runtime_parity_assertions", {})
    expected_keys = (
        "same_command_tree_where_parity_exists",
        "same_exit_codes_where_parity_exists",
        "same_output_envelopes_where_parity_exists",
    )
    missing = [k for k in expected_keys if k not in assertions]
    if missing:
        return False, f"missing runtime parity assertions: {', '.join(missing)}"
    return True, "runtime parity assertions present in current state artifact"


def build_report() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    for name, fn in (
        ("python_bridge_same_core_entrypoint", check_python_bridge_uses_core_entrypoint),
        ("python_bridge_no_separate_routing", check_python_bridge_has_no_separate_routing),
        ("python_bridge_no_separate_exit_mapping", check_python_bridge_has_no_separate_exit_mapping),
        ("python_bridge_no_separate_output_semantics", check_python_bridge_has_no_separate_output_semantics),
        ("binary_crate_delegates_behavior_to_core", check_bin_crate_logic_boundary),
        ("scripts_do_not_bypass_supported_rust_entrypoint", check_scripts_do_not_bypass_supported_rust_entrypoint),
        ("package_metadata_points_to_bijux", check_package_metadata_points_to_bijux),
        ("runtime_identity_command_visible", check_runtime_identity_command_visible),
        ("one_law_document_present", check_one_law_document_exists),
        ("runtime_parity_assertions_present", load_parity_assertions),
    ):
        ok, detail = fn()
        checks.append({"name": name, "ok": ok, "detail": detail})

    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/check_runtime_unity.py",
        "checks": checks,
        "ok": all(bool(c["ok"]) for c in checks),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true", help="return non-zero if any check fails")
    args = parser.parse_args()

    report = build_report()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")

    if args.enforce and not report["ok"]:
        for check in report["checks"]:
            if not check["ok"]:
                print(f"RUNTIME UNITY CHECK FAILED: {check['name']}: {check['detail']}")
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
