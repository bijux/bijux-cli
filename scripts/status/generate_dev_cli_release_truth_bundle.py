#!/usr/bin/env python3
"""Generate dev-cli release truth bundle from canonical release commands."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS = ROOT / "artifacts" / "status"

COMMANDS = {
    "status": ["dev", "cli", "release", "status"],
    "evidence": ["dev", "cli", "release", "evidence"],
    "readiness": ["dev", "cli", "release", "readiness"],
    "diff": ["dev", "cli", "release", "diff"],
    "gaps": ["dev", "cli", "release", "gaps"],
    "behavior_changes": ["dev", "cli", "release", "behavior-changes"],
    "intentional_differences": ["dev", "cli", "release", "intentional-differences"],
    "unresolved_gaps": ["dev", "cli", "release", "unresolved-gaps"],
    "compatibility_leftovers": ["dev", "cli", "release", "compatibility-leftovers"],
}


def run_json(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "bijux-cli", "--", *args, "--format", "json", "--no-pretty"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(proc.stdout or "{}")


def main() -> int:
    STATUS.mkdir(parents=True, exist_ok=True)
    bundle = {"source": "dev cli release *", "reports": {}}
    for key, command in COMMANDS.items():
        payload = run_json(command)
        bundle["reports"][key] = payload

    gaps = bundle["reports"].get("gaps", {})
    unresolved = gaps.get("unresolved_gaps", []) if isinstance(gaps, dict) else []
    missing = gaps.get("missing_evidence", []) if isinstance(gaps, dict) else []
    bundle["summary"] = {
        "unresolved_gaps": len(unresolved) if isinstance(unresolved, list) else 0,
        "missing_evidence": len(missing) if isinstance(missing, list) else 0,
    }

    out = STATUS / "dev_cli_release_truth_bundle.json"
    out.write_text(json.dumps(bundle, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
