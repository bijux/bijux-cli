#!/usr/bin/env python3
"""Generate flaky test labeling artifact for CI visibility."""

from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "artifacts" / "status" / "flaky_tests.json"


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT)).replace("\\", "/")


def main() -> int:
    rows = []
    for path in sorted(ROOT.rglob("*.rs")):
        if "tests" not in path.parts or "target" in path.parts:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for m in re.finditer(r"#\[ignore(?:\s*=\s*\"([^\"]+)\")?\]", text):
            reason = (m.group(1) or "").lower()
            if "flaky" in reason:
                rows.append({"path": rel(path), "label": "flaky", "reason": reason or "flaky"})

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "generator": "scripts/status/generate_flaky_test_labels.py",
        "label": "flaky",
        "count": len(rows),
        "tests": rows,
        "policy": "no flaky test may be silently ignored; each flaky marker requires remediation tracking",
    }

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
