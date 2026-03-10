#!/usr/bin/env python3
"""Generate repo/docs/scripts/crate-health truth and drift artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli-core", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def write_json(name: str, payload: dict) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    repo = run_json(["dev", "cli", "repo", "health"])
    docs = run_json(["dev", "cli", "docs-audit"])
    scripts = run_json(["dev", "cli", "script-audit"])
    crate_health = run_json(["dev", "cli", "crate-health"])

    checks = {
        "repo_health_payload_present": isinstance(repo.get("repo_health"), dict),
        "docs_payload_present": isinstance(docs.get("docs"), list),
        "scripts_payload_present": isinstance(scripts.get("scripts"), list),
        "crate_metrics_payload_present": isinstance(crate_health.get("crate_metrics"), dict),
        "docs_audit_summary_present": isinstance(docs.get("docs_audit"), dict),
        "script_audit_remaining_signal_present": scripts.get("remaining_script_only_behaviors") is not None,
        "crate_health_dependency_edges_present": isinstance(crate_health.get("dependency_edges"), list),
        "crate_health_public_api_inventory_present": isinstance(crate_health.get("public_api_by_crate"), dict),
        "repo_health_stale_generated_signal_present": isinstance(
            repo.get("repo_health", {}).get("generated", {}).get("stale_generated_artifacts"),
            list,
        )
        or isinstance(
            repo.get("repo_health", {}).get("stale", {}).get("stale_generated_artifacts"),
            list,
        ),
    }
    drift = [name for name, ok in checks.items() if not ok]

    write_json(
        "repo_docs_scripts_crate_health_artifact.json",
        {
            "scope": "repo/docs/scripts/crate-health truth",
            "generator": "scripts/status/generate_dev_cli_repo_docs_scripts_crate_health_reports.py",
            "checks": checks,
            "status": "complete" if all(checks.values()) else "partial",
        },
    )
    write_json(
        "repo_docs_scripts_crate_health_drift_artifact.json",
        {
            "scope": "repo/docs/scripts/crate-health drift",
            "generator": "scripts/status/generate_dev_cli_repo_docs_scripts_crate_health_reports.py",
            "drift_checks": drift,
            "drift_count": len(drift),
            "status": "clean" if not drift else "drift",
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
