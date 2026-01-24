# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Gate for pip-audit results.

Reads a pip-audit JSON report, filters out ignored vulnerability IDs (and aliases),
prints a concise, deterministic summary, and exits non-zero when problems remain.

Environment variables:
  PIPA_JSON             Path to pip-audit JSON (default: artifacts_pages/security/pip-audit.json)
  SECURITY_IGNORE_IDS   Space-separated list of IDs to ignore (e.g., "CVE-2023-1234 GHSA-xxxx")
  SECURITY_STRICT       "1" to fail when report is missing/unreadable or vulns remain; else soft-pass
"""

from __future__ import annotations

import json
import os
import sys
from typing import Any, Dict, Iterable, List, Sequence, Set, Tuple

REPORT_PATH = os.getenv("PIPA_JSON", "artifacts_pages/security/pip-audit.json")
IGNORE_IDS: Set[str] = set(filter(None, os.getenv("SECURITY_IGNORE_IDS", "").split()))
IS_STRICT = os.getenv("SECURITY_STRICT", "1") == "1"


def _load_report(path: str) -> List[Dict[str, Any]]:
    """Load pip-audit JSON, tolerating both list and {dependencies: []} schemas."""
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        msg = f"ERROR: pip-audit JSON missing/unreadable at '{path}': {e!s}"
        if IS_STRICT:
            print(msg)
            sys.exit(2)
        print(msg + " (non-strict: continuing with empty report)")
        return []

    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        deps = data.get("dependencies", [])
        if isinstance(deps, list):
            return deps
    msg = f"ERROR: unexpected report format in '{path}'"
    if IS_STRICT:
        print(msg)
        sys.exit(2)
    print(msg + " (non-strict: continuing with empty report)")
    return []


def _all_ids(vuln: Dict[str, Any]) -> Set[str]:
    """Collect primary id and aliases (if any) into a set."""
    ids: Set[str] = set()
    vid = vuln.get("id")
    if isinstance(vid, str) and vid:
        ids.add(vid)
    aliases = vuln.get("aliases") or []
    if isinstance(aliases, list):
        for a in aliases:
            if isinstance(a, str) and a:
                ids.add(a)
    return ids


def _primary_id(ids: Iterable[str]) -> str:
    """Deterministic representative ID for display."""
    try:
        return sorted(ids)[0]
    except IndexError:
        return "?"


def _fmt_table(rows: Sequence[Sequence[str]], header: Sequence[str]) -> str:
    """Simple ASCII table with dynamic widths."""
    widths = [len(h) for h in header]
    for r in rows:
        for i, cell in enumerate(r):
            if i < len(widths):
                widths[i] = max(widths[i], len(cell))
            else:
                widths.append(len(cell))

    def fmt_row(cols: Sequence[str]) -> str:
        return "  ".join(c.ljust(widths[i]) for i, c in enumerate(cols))

    sep = "  ".join("-" * w for w in widths)
    out = [fmt_row(header), sep]
    out.extend(fmt_row(r) for r in rows)
    return "\n".join(out)


def main() -> None:
    if IGNORE_IDS:
        ids = " ".join(sorted(IGNORE_IDS))
        print(f"INFO: ignoring IDs/aliases: {ids}")

    deps = _load_report(REPORT_PATH)
    if not deps:
        print("OK: no dependencies in report (or empty after parsing).")
        sys.exit(0)

    remaining: List[Tuple[str, str, str, str]] = []
    ignored_count = 0

    for d in deps:
        name = d.get("name", "?")
        ver = d.get("version", "?")
        vulns = d.get("vulns") or []
        if not isinstance(vulns, list):
            continue
        for v in vulns:
            ids = _all_ids(v)
            if ids & IGNORE_IDS:
                ignored_count += 1
                continue
            fix_versions = v.get("fix_versions") or []
            if not isinstance(fix_versions, list):
                fix_versions = []
            fix = ", ".join(fix_versions) if fix_versions else "-"
            remaining.append((str(name), str(ver), _primary_id(ids), fix))

    if ignored_count:
        print(
            f"INFO: {ignored_count} vulnerability instance(s) matched ignore list and were skipped."
        )

    if not remaining:
        print("OK: 0 vulnerabilities remain after ignores.")
        sys.exit(0)

    remaining.sort(key=lambda r: (r[0], r[2], r[1]))

    header = ("Package", "Version", "ID", "FixVersions")
    table = _fmt_table(remaining, header)
    print(
        f"FAIL: {len(remaining)} vulnerability instance(s) remain after ignores.\n{table}"
    )

    if IS_STRICT:
        print(f"STRICT: failing due to remaining vulnerabilities. See {REPORT_PATH}")
        sys.exit(1)
    else:
        print(
            f"NON-STRICT: not failing despite remaining vulnerabilities. See {REPORT_PATH}"
        )
        sys.exit(0)


if __name__ == "__main__":
    main()
