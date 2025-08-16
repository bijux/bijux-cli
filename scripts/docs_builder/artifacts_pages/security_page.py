# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Builds the documentation page for security artifacts.

This module defines `SecurityArtifactPage`, which implements the
`StandardArtifactPage` base class to parse and display reports from `bandit`
(code security) and `pip-audit` (dependency security).

Module Constants:
    SEC_DIR: The directory where security artifact logs are stored.
    ORDER: A list defining the presentation order of artifacts on the page.
    BLURBS: A dictionary mapping artifact filenames to their descriptions
        and a summary of a "good" result.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Iterable

from scripts.docs_builder.artifacts_pages.base import StandardArtifactPage
from scripts.docs_builder.artifacts_pages.base import Bullet
from scripts.docs_builder.helpers import anchor_for
from scripts.docs_builder.helpers import indent

SEC_DIR = Path("artifacts/security")

ORDER = ["bandit.txt", "bandit.json", "pip-audit.txt", "pip-audit.json"]

BLURBS = {
    "bandit.txt": ("Bandit screen output (Python security lint).", "0 issues (or all below allowed severity)."),
    "bandit.json": ("Bandit JSON report with per-issue details.", "0 total issues; no HIGH/MED severities."),
    "pip-audit.txt": (
        "pip-audit screen summary (dependency CVE scan; gated by helper script).",
        "0 vulnerable packages.",
    ),
    "pip-audit.json": ("pip-audit JSON report (one run; helper script gates separately).", "0 vulnerabilities reported."),
}


def _bandit_json(fp: Path) -> tuple[int, dict[str, int]]:
    """Parses a bandit JSON report to count issues by severity.

    Args:
        fp: The path to the bandit JSON file.

    Returns:
        A tuple containing:
            - An integer for the total number of issues found.
            - A dictionary with counts for 'HIGH', 'MEDIUM', and 'LOW'
              severity issues.
        Returns (0, {'HIGH': 0, ...}) if the file cannot be read or parsed.
    """
    sev = {"HIGH": 0, "MEDIUM": 0, "LOW": 0}
    try:
        data = json.loads(fp.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return 0, sev

    for r in data.get("results") or []:
        s = (r.get("issue_severity") or "").upper()
        if s in sev:
            sev[s] += 1
    return sum(sev.values()), sev


def _pip_audit_json(fp: Path) -> tuple[int, int, int]:
    """Parses a pip-audit JSON report to count vulnerabilities.

    Handles different JSON output formats that pip-audit may produce.

    Args:
        fp: The path to the pip-audit JSON file.

    Returns:
        A tuple containing:
            - The total number of packages scanned.
            - The number of packages found to have vulnerabilities.
            - The total number of vulnerabilities found across all packages.
        Returns (0, 0, 0) if the file cannot be read or parsed.
    """
    try:
        data = json.loads(fp.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return 0, 0, 0

    deps = data.get("dependencies") if isinstance(data, dict) else data
    if not isinstance(deps, list):
        return 0, 0, 0

    pkgs = len(deps)
    vuln_pkgs = sum(1 for d in deps if d.get("vulns"))
    vulns = sum(len(d.get("vulns") or []) for d in deps)
    return pkgs, vuln_pkgs, vulns


class SecurityArtifactPage(StandardArtifactPage):
    """Generates the documentation page for security scan artifacts."""

    out_md = Path("artifacts/security.md")

    def title(self) -> str:
        """Returns the main title for the security artifacts page."""
        return "Security Artifacts"

    def intro(self) -> str:
        """Returns the introductory text for the security artifacts page."""
        return (
            "Security reports for **code** and **dependencies**, produced by `make security`:\n\n"
            "- **Bandit** — source-level checks (`bandit -r`, text + JSON)\n"
            "- **pip-audit** — dependency CVE scan (JSON once; gated/pretty text via `scripts/helper_pip_audit.py`)\n"
            "\n"
            "Makefile knobs: `SECURITY_PATHS`, `SECURITY_IGNORE_IDS`, `SECURITY_STRICT`, `BANDIT_EXCLUDES`.\n"
        )

    def iter_items(self) -> Iterable[tuple[str, Path]]:
        """Yields the security artifact files to be documented."""
        return [(name, SEC_DIR / name) for name in ORDER if (SEC_DIR / name).exists()]

    def bullet_for(self, label: str, path: Path, content: str) -> Bullet:
        """Builds a detailed summary bullet for a specific security artifact.

        This method calls JSON parsers for `bandit.json` and `pip-audit.json`
        to generate a rich summary of the findings.

        Args:
            label: The name of the artifact (e.g., "bandit.json").
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A `Bullet` object populated with a title, summary, and usage guidance.
        """
        desc, good_template = BLURBS.get(label, ("Report", ""))
        extra = ""

        if label == "bandit.json":
            total, sev = _bandit_json(path)
            extra = f" — {total} issues (H:{sev['HIGH']} M:{sev['MEDIUM']} L:{sev['LOW']})"
        elif label == "pip-audit.json":
            pkgs, vuln_pkgs, vulns = _pip_audit_json(path)
            extra = f" — {vulns} vulns across {vuln_pkgs}/{pkgs} packages"

        title = f"[{label}](#{anchor_for(label)}) — {desc}"
        good_line = f"{good_template}{extra}"

        usage = {
            "bandit.txt": "Fix HIGH/MED first; use `# nosec` only with justification; re-run `make security-bandit`.",
            "bandit.json": "Prioritize HIGH/MED; adjust `BANDIT_EXCLUDES` or inline ignores sparingly.",
            "pip-audit.txt": "Upgrade/patch; respect `SECURITY_IGNORE_IDS`; `SECURITY_STRICT=1` gates CI.",
            "pip-audit.json": "Open fixes or pin versions; re-run after dependency changes or `pip-compile`.",
        }.get(label)

        return Bullet(title=title, good=good_line, usage=usage)

    def detail_for(self, label: str, path: Path, content: str) -> str:
        """Creates the detailed description for a given security artifact.

        This method generates a Markdown 'info' block with details about the
        artifact. For JSON reports, it calls the appropriate parser to include a
        formatted summary of the findings. It also provides extra context, such
        as how `pip-audit`'s CI gating is handled by a helper script.

        Args:
            label: The name of the artifact (e.g., "bandit.json").
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A formatted Markdown string for the artifact's detail section.
        """
        desc, good = BLURBS.get(label, ("Report", ""))
        lines = [f"**What it is:** {desc}", f"**What good looks like:** {good}"]

        if label == "bandit.json":
            total, sev = _bandit_json(path)
            lines.append(
                f"**Summary:** total {total} issues — HIGH {sev['HIGH']}, "
                f"MEDIUM {sev['MEDIUM']}, LOW {sev['LOW']}."
            )
        elif label == "pip-audit.json":
            pkgs, vuln_pkgs, vulns = _pip_audit_json(path)
            lines.append(
                f"**Summary:** scanned {pkgs} packages; {vuln_pkgs} packages vulnerable; "
                f"{vulns} total vulnerabilities."
            )
            lines.append(
                "Console gating is done by `scripts/helper_pip_audit.py` (respects `SECURITY_IGNORE_IDS` and "
                "`SECURITY_STRICT`)."
            )

        return '!!! info "About this artifact"\n\n' + indent("\n".join(lines) + "\n")
