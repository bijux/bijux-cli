#!/usr/bin/env python3
"""Generate runtime dev-leakage audit report across runtime crates."""

from __future__ import annotations

import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = REPO_ROOT / "artifacts" / "status"

RUNTIME_CRATE_SRCS = {
    "bijux-cli-core": REPO_ROOT / "crates" / "bijux-cli-core" / "src",
    "bijux-cli-routing": REPO_ROOT / "crates" / "bijux-cli-routing" / "src",
    "bijux-cli-output": REPO_ROOT / "crates" / "bijux-cli-output" / "src",
    "bijux-cli-install": REPO_ROOT / "crates" / "bijux-cli-install" / "src",
    "bijux-cli-plugin": REPO_ROOT / "crates" / "bijux-cli-plugin" / "src",
    "bijux-cli-python": REPO_ROOT / "crates" / "bijux-cli-python" / "src",
}


def iter_rs_files(root: Path) -> list[Path]:
    files = sorted(p for p in root.rglob("*.rs") if p.is_file())
    return files


def main() -> int:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    rows: list[dict[str, object]] = []

    for crate, src_root in RUNTIME_CRATE_SRCS.items():
        text = "\n".join(path.read_text(encoding="utf-8") for path in iter_rs_files(src_root))
        bijux_dev_cli_imports = text.count("bijux_dev_cli")
        dev_cli_literals = text.count("dev cli")
        route_audit_assembly = text.count("route_audit_report(")
        report_builder_calls = text.count("build_report(")
        # core is the dispatcher and delegates report builders by design.
        if crate == "bijux-cli-core":
            report_builder_calls = 0
            bijux_dev_cli_imports = 0
        # routing owns command identity; command literals are expected.
        if crate == "bijux-cli-routing":
            dev_cli_literals = 0
        leakage_score = (
            bijux_dev_cli_imports + dev_cli_literals + route_audit_assembly + report_builder_calls
        )
        rows.append(
            {
                "crate": crate,
                "bijux_dev_cli_imports": bijux_dev_cli_imports,
                "dev_cli_literals": dev_cli_literals,
                "route_audit_assembly_calls": route_audit_assembly,
                "report_builder_calls_outside_core_exception": report_builder_calls,
                "leakage_score": leakage_score,
            }
        )

    total_leakage = sum(int(row["leakage_score"]) for row in rows)
    report = {
        "scope": "runtime dev leakage",
        "status": "ok" if total_leakage == 0 else "degraded",
        "total_leakage_score": total_leakage,
        "crates": rows,
        "rules": [
            "runtime crates stay focused on runtime law",
            "maintainer workflow report assembly belongs in bijux-dev-cli",
            "runtime crates do not import bijux-dev-cli directly",
        ],
    }

    (STATUS_DIR / "runtime_dev_leakage_report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
