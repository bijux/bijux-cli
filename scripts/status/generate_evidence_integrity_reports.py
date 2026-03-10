#!/usr/bin/env python3
"""Generate evidence integrity and config ownership truth artifacts."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def write(name: str, payload: dict) -> None:
    out = STATUS / name
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)

    evidence_audit = run_json(["dev", "cli", "evidence", "audit"])
    evidence_map = run_json(["dev", "cli", "evidence", "command-map"])
    parity_map = run_json(["dev", "cli", "evidence", "parity-map"])

    write(
        "evidence_coverage_report.json",
        {"records": evidence_audit.get("coverage_report", []), "source": "dev cli evidence audit"},
    )
    write(
        "evidence_integrity_artifact.json",
        {
            "generator": "scripts/status/generate_evidence_integrity_reports.py",
            "scope": "evidence integrity",
            "checks": {
                "invalid_ids": evidence_audit.get("invalid_ids", []),
                "missing_artifact_links": evidence_audit.get("missing_artifact_links", []),
                "orphan_report": evidence_audit.get("orphan_report", []),
                "claims_without_evidence_report": evidence_audit.get("claims_without_evidence_report", []),
            },
            "status": (
                "complete"
                if (
                    not evidence_audit.get("invalid_ids", [])
                    and not evidence_audit.get("missing_artifact_links", [])
                    and not evidence_audit.get("orphan_report", [])
                    and not evidence_audit.get("claims_without_evidence_report", [])
                )
                else "partial"
            ),
        },
    )
    write(
        "orphan_evidence_report.json",
        {"records": evidence_audit.get("orphan_report", []), "source": "dev cli evidence audit"},
    )
    write(
        "orphan_evidence_artifact.json",
        {
            "generator": "scripts/status/generate_evidence_integrity_reports.py",
            "scope": "orphan evidence",
            "records": evidence_audit.get("orphan_report", []),
            "count": len(evidence_audit.get("orphan_report", [])),
            "status": "clean" if not evidence_audit.get("orphan_report", []) else "drift",
        },
    )
    write(
        "claim_without_evidence_report.json",
        {
            "records": evidence_audit.get("claims_without_evidence_report", []),
            "source": "dev cli evidence audit",
        },
    )
    write("evidence_command_map_report.json", evidence_map)
    write("evidence_parity_map_report.json", parity_map)

    rust_owner = run_json(["dev", "cli", "config", "rust-owner"])
    python_owner = run_json(["dev", "cli", "config", "python-owner"])
    ownership = run_json(["dev", "cli", "config", "ownership"])
    drift = run_json(["dev", "cli", "config", "drift"])
    shape = run_json(["dev", "cli", "config", "shape"])
    evidence_link = run_json(["dev", "cli", "config", "evidence-map"])

    write("config_owners_by_layer_report.json", {"rust": rust_owner, "python": python_owner})
    write(
        "config_file_schema_owners_report.json",
        {"owners": ownership.get("owners", {}), "schemas": shape.get("schemas", [])},
    )
    write(
        "config_python_compatibility_shims_report.json",
        {"compatibility_shims": ownership.get("compatibility_shims", [])},
    )
    write("config_rust_sources_report.json", {"sources": shape.get("sources", [])})
    write("config_precedence_proofs_report.json", {"precedence_proofs": shape.get("precedence_proofs", [])})
    write("config_mutation_rollback_proofs_report.json", {"rollback_proofs": shape.get("rollback_proofs", [])})
    write("config_corruption_evidence_report.json", {"corruption_evidence": shape.get("corruption_evidence", [])})
    write("config_owner_drift_report.json", drift)
    write("config_evidence_link_report.json", evidence_link)
    write(
        "config_ownership_truth.json",
        {
            "owners": ownership.get("owners", {}),
            "schemas": shape.get("schemas", []),
            "compatibility_shims": ownership.get("compatibility_shims", []),
            "sources": shape.get("sources", []),
            "precedence_proofs": shape.get("precedence_proofs", []),
            "rollback_proofs": shape.get("rollback_proofs", []),
            "corruption_evidence": shape.get("corruption_evidence", []),
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
