#!/usr/bin/env python3
"""Generate cleanup evidence and retention-policy status artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


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


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS.mkdir(parents=True, exist_ok=True)
    (STATUS / name).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    generated_at = stable_generated_at()

    deleted_docs = [
        "docs/architecture/newly-ported-command-parity.md",
        "docs/architecture/next-five-command-priorities.md",
        "docs/architecture/safe-improvements-after-parity.md",
    ]
    deleted_snapshot_files = [
        "artifacts/python-behavior/golden/config/config_get_sample.json",
        "artifacts/python-behavior/golden/config/config_set_sample.json",
        "artifacts/python-behavior/golden/config/config_unset_sample.json",
    ]
    deleted_artifacts = [
        "artifacts/python-behavior/golden/config/capture-summary.json",
        "artifacts/python-behavior/golden/config/config_clear.json",
        "artifacts/python-behavior/golden/config/config_export_json.json",
    ]

    policy_files = {
        "artifact_retention": "docs/architecture/artifact-retention-policy.md",
        "snapshot_retention": "docs/architecture/snapshot-retention-policy.md",
        "document_retention": "docs/architecture/document-retention-policy.md",
    }

    write_json(
        "docs_unreferenced_candidates.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cleanup_reports.py",
            "deleted": deleted_docs,
            "criteria": [
                "not linked by README, command reference, or contributor flow",
                "historical progress reporting rather than durable law",
            ],
        },
    )

    write_json(
        "stale_snapshot_candidates.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cleanup_reports.py",
            "deleted": deleted_snapshot_files,
            "criteria": [
                "legacy python-behavior captures no longer tied to live rust command snapshots",
                "not consumed by CI upload, release evidence, or tests",
            ],
        },
    )

    write_json(
        "dead_generated_artifact_candidates.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cleanup_reports.py",
            "deleted": deleted_artifacts,
            "criteria": [
                "runtime lock and temp files in artifact tree are not evidence artifacts",
                "legacy python behavior captures not consumed by CI upload, release evidence, or status reports",
            ],
        },
    )

    write_json(
        "cleanup_report.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_cleanup_reports.py",
            "scope": "761-780 cleanup and retention hardening",
            "deleted": {
                "docs": deleted_docs,
                "snapshot_artifacts": deleted_snapshot_files,
                "dead_generated_artifacts": deleted_artifacts,
            },
            "policies": policy_files,
            "rules": [
                "reject keep-just-in-case for stale prose",
                "reject keep-just-in-case for stale snapshots",
                "reject keep-just-in-case for dead generated artifacts",
                "cleanup is ongoing release-by-release work",
            ],
            "status": "complete",
        },
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
