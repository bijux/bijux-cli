# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Builds the documentation page for code quality artifacts.

This module defines `QualityArtifactPage`, which implements the
`StandardArtifactPage` base class to parse and display reports from tools like
`deptry`, `interrogate`, `REUSE`, and `vulture`.

Module Constants:
    QUALITY_DIR: The directory where quality artifact logs are stored.
    ORDER: A list defining the primary presentation order of artifacts on the page.
    BLURBS: A dictionary mapping artifact filenames to their descriptions
        and a summary of a "good" result.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable

from scripts.docs_builder.artifacts_pages.base import StandardArtifactPage
from scripts.docs_builder.artifacts_pages.base import Bullet
from scripts.docs_builder.helpers import anchor_for
from scripts.docs_builder.helpers import indent

QUALITY_DIR = Path("artifacts/quality")

ORDER = [
    "deptry.log",
    "interrogate.full.txt",
    "interrogate.offenders.txt",
    "reuse.log",
    "vulture.log",
    "_passed",
]

BLURBS = {
    "deptry.log": ("Dependency hygiene (unused/missing/transitive).", "No missing deps; minimal unused/transitive."),
    "interrogate.full.txt": ("Docstring coverage summary.", "≥ target coverage; PASSED."),
    "interrogate.offenders.txt": ("Files/symbols missing required docstrings.", "Empty or tiny list."),
    "reuse.log": ("REUSE compliance (license metadata).", "Compliant with 0 errors."),
    "vulture.log": ("Dead-code finder.", "No actionable dead code."),
    "_passed": ("Suite sentinel.", "Present with OK marker."),
}


def _pct(text: str) -> float | None:
    """Extracts the first percentage value from a string.

    Args:
        text: The string to search.

    Returns:
        The percentage as a float, or None if no percentage is found.
    """
    m = re.search(r"(\d+(?:\.\d+)?)\s*%", text)
    return float(m.group(1)) if m else None


def _deptry(text: str) -> tuple[int, int, int]:
    """Parses deptry log content to count dependency issues.

    Args:
        text: The raw string content of the deptry log file.

    Returns:
        A tuple containing the counts of (missing, unused, transitive)
        dependencies found.
    """
    low = text.lower()

    def grab(pattern: str) -> int:
        """Counts occurrences of a pattern."""
        return sum(1 for _ in re.finditer(pattern, low))

    missing = grab(r"\bmissing (?:dep|dependency|dependencies|import)")
    unused = grab(r"\bunused (?:dep|dependency|dependencies|import)")
    trans = grab(r"\btransitive (?:dep|dependency|dependencies|import)")
    return missing, unused, trans


def _reuse_status(text: str) -> str | None:
    """Parses a REUSE log to determine the compliance status.

    Args:
        text: The raw string content of the REUSE log file.

    Returns:
        'compliant', 'non-compliant', or None if the status cannot be determined.
    """
    low = text.lower()
    if "not compliant" in low:
        return "non-compliant"
    if "compliant" in low:
        return "compliant"
    return None


def _vulture_items(text: str) -> int:
    """Parses a vulture log to count the number of suspected dead code items.

    Args:
        text: The raw string content of the vulture log file.

    Returns:
        The number of items reported by vulture.
    """
    return sum(1 for ln in text.splitlines() if re.search(r":[0-9]+:", ln))


def _offenders_count(text: str) -> int:
    """Parses an interrogate offenders report to count listed items.

    Args:
        text: The raw string content of the interrogate offenders file.

    Returns:
        The number of offending files or symbols listed.
    """
    return sum(1 for ln in text.splitlines() if ln.strip().startswith("- "))


class QualityArtifactPage(StandardArtifactPage):
    """Generates the documentation page for code quality artifacts."""

    out_md = Path("artifacts/quality.md")

    def title(self) -> str:
        """Returns the main title for the quality artifacts page."""
        return "Quality Artifacts"

    def intro(self) -> str:
        """Returns the introductory text for the quality artifacts page."""
        return (
            "Reports that guard broader code quality: dependencies, docstring coverage, licensing, and dead code.\n\n"
            "- **deptry**: unused / missing / transitive imports\n"
            "- **interrogate**: docstring coverage (full table + offenders)\n"
            "- **REUSE**: license & SPDX compliance\n"
            "- **vulture**: likely dead code\n"
        )

    def iter_items(self) -> Iterable[tuple[str, Path]]:
        """Yields the quality artifact files to be documented.

        It first yields files listed in the `ORDER` constant that exist on disk,
        maintaining that specific order. It then yields any other files found in
        the quality artifacts directory, sorted alphabetically by name.

        Yields:
            An iterable of (label, path) tuples for each artifact.
        """
        if not QUALITY_DIR.is_dir():
            return []
        files = [QUALITY_DIR / n for n in ORDER if (QUALITY_DIR / n).exists()]
        seen = {p.name for p in files}
        files += sorted(p for p in QUALITY_DIR.glob("*") if p.is_file() and p.name not in seen)
        return [(p.name, p) for p in files]

    def bullet_for(self, label: str, path: Path, content: str) -> Bullet:
        """Builds a detailed summary bullet for a specific quality artifact.

        This method dynamically constructs the bullet point by:
        1.  Looking up the artifact's base description from the `BLURBS` constant.
        2.  Selecting the appropriate parsing function based on the artifact's
            label (filename) to extract metrics from the log content.
        3.  Formatting these metrics into a detailed summary string.
        4.  Assigning a relevant "How to use" tip.

        Args:
            label: The filename of the artifact (e.g., "deptry.log").
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A `Bullet` object populated with the title, a summary of the result,
            and usage guidance.
        """
        desc, good = BLURBS.get(label, ("Report", ""))
        extra = ""

        if label == "deptry.log":
            m, u, t = _deptry(content)
            extra = f"missing: {m}, unused: {u}, transitive: {t}"
        elif label == "interrogate.full.txt":
            pc = _pct(content)
            if pc is not None:
                extra = f"coverage ~{pc:.1f}%"
        elif label == "interrogate.offenders.txt":
            n = _offenders_count(content)
            extra = f"offenders: {n}"
        elif label == "reuse.log":
            status = _reuse_status(content)
            if status:
                extra = status
        elif label == "vulture.log":
            extra = f"suspected items: {_vulture_items(content)}"
        elif label == "_passed":
            extra = "OK"

        title = f"[{label}](#{anchor_for(label)}) — {desc}"
        good_line = f"{good}" + (f" — {extra}" if extra else "")

        usage = {
            "deptry.log": "Remove unused deps; add direct deps for imported transitive modules.",
            "interrogate.full.txt": "Add concise docstrings where missing; enforce threshold in CI.",
            "interrogate.offenders.txt": "Open listed files and add minimal docstrings.",
            "reuse.log": "Ensure SPDX headers/REUSE metadata per file; add/maintain `REUSE.toml`.",
            "vulture.log": "Delete true dead code; whitelist false positives.",
            "_passed": "All checks green.",
        }.get(label)

        return Bullet(title=title, good=good_line, usage=usage)

    def detail_for(self, label: str, path: Path, content: str) -> str:
        """Creates the detailed description for a given quality artifact.

        This method generates a Markdown 'info' block containing the purpose
        of the tool and a description of a desirable outcome, based on the
        `BLURBS` constant.

        Args:
            label: The filename of the artifact.
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A formatted Markdown string for the artifact's detail section.
        """
        desc, good = BLURBS.get(label, ("Report", ""))
        lines = [f"**What it is:** {desc}", f"**What good looks like:** {good}"]
        return '!!! info "About this report & how to act on it"\n\n' + indent("\n".join(lines) + "\n")
