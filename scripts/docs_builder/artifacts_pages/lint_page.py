# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Builds the documentation page for linting and static analysis artifacts.

This module defines the `LintArtifactPage`, which implements the
`StandardArtifactPage` base class to parse and display results from tools like
Ruff, Mypy, and Pyright. It uses a set of specialized parser functions to extract
key metrics from the raw log files of each tool.

Module Constants:
    LINT_DIR: The directory where lint artifact logs are stored.
    ORDER: A list defining the presentation order of artifacts on the page,
        matching the execution order in the Makefile.
    BLURBS: A dictionary mapping artifact filenames to their descriptions
        and a summary of what a "good" result looks like.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable

from scripts.docs_builder.artifacts_pages.base import StandardArtifactPage
from scripts.docs_builder.artifacts_pages.base import anchor_for
from scripts.docs_builder.artifacts_pages.base import Bullet
from scripts.docs_builder.helpers import indent

LINT_DIR = Path("artifacts/lint")

ORDER = [
    "ruff-format.log",
    "ruff.log",
    "mypy.log",
    "pyright.log",
    "codespell.log",
    "radon.log",
    "pydocstyle.log",
    "pytype.log",
    "_passed",
]

BLURBS = {
    "ruff.log": ("Python linter (style + correctness).", "0 violations (or only nits)."),
    "ruff-format.log": ("Formatter check (`ruff format --check`).", "No changes needed."),
    "codespell.log": ("Spelling checker for identifiers, comments, docs.", "Empty log (no typos)."),
    "mypy.log": ("Static type-checker (mypy).", "Success: no issues found."),
    "pyright.log": ("Static type-checker (pyright).", "0 errors, 0 warnings."),
    "pytype.log": ("Type inference + checking (pytype).", "No errors."),
    "pydocstyle.log": ("Google-style docstring conventions (pydocstyle).", "0 violations (Google convention)."),
    "radon.log": ("Complexity & maintainability (CC, MI).", "Mostly A; MI ≥ 70; nothing above thresholds."),
    "_passed": ("Suite sentinel.", "Present with OK marker."),
}


def _parse_ruff(text: str) -> int:
    """Parses ruff log content to count the number of issues.

    It first attempts to count specific issue locations (e.g.,
    'file:line:col: F401') and falls back to counting any occurrences of ruff
    rule codes (e.g., 'F401') as a less precise measure.

    Args:
        text: The raw string content of the ruff log file.

    Returns:
        The total number of ruff issues found.
    """
    hits = len(re.findall(r":\d+:\d+:\s*[A-Z]\d{3}", text))
    return hits or len(re.findall(r"\b[A-Z]\d{3}\b", text))


def _parse_ruff_format(text: str) -> tuple[int, int]:
    """Parses a `ruff format --check` log to count needed changes.

    Args:
        text: The raw string content of the ruff-format log file.

    Returns:
        A tuple containing:
            - The number of files that "would be reformatted".
            - The number of files that were "reformatted" (in non-check mode).
    """
    would = len(re.findall(r"would be reformatted", text, re.I))
    changed = len(re.findall(r"\breformatted\b", text, re.I))
    return would, changed


def _parse_codespell(text: str) -> int:
    """Parses codespell log content to count the number of typos.

    It counts lines that match the typical codespell output format and also
    looks for a final summary line like "X errors found".

    Args:
        text: The raw string content of the codespell log file.

    Returns:
        The total number of typos found.
    """
    line_hits = sum(1 for ln in text.splitlines() if re.search(r":\d+(?::\d+)?:.*(?:->|==>)", ln))
    m = re.search(r"(\d+)\s+errors?\s+found", text, re.I)
    return max(line_hits, int(m.group(1))) if m else line_hits


def _parse_mypy(text: str) -> tuple[bool, int]:
    """Parses mypy log content to check for success or count errors.

    Args:
        text: The raw string content of the mypy log file.

    Returns:
        A tuple containing:
            - A boolean indicating if the check was successful.
            - An integer count of the errors found.
    """
    if re.search(r"Success:\s*no issues found", text, re.I):
        return True, 0
    m = re.search(r"Found\s+(\d+)\s+error", text, re.I)
    return False, (int(m.group(1)) if m else 0)


def _parse_pyright(text: str) -> tuple[int, int]:
    """Parses pyright log content to count errors and warnings.

    Args:
        text: The raw string content of the pyright log file.

    Returns:
        A tuple containing the number of errors and warnings, respectively.
    """
    m = re.search(r"Found\s+(\d+)\s+errors?,\s+(\d+)\s+warnings?", text, re.I)
    return (int(m.group(1)), int(m.group(2))) if m else (0, 0)


def _pytype_skipped(text: str) -> bool:
    """Checks if the pytype log indicates that the check was skipped.

    The Makefile that runs the check may write a specific sentinel message if
    the Python version is unsupported.

    Args:
        text: The raw string content of the pytype log file.

    Returns:
        True if the log contains the 'skipped' message, False otherwise.
    """
    return "Pytype skipped on Python" in text


def _parse_pytype(text: str) -> int:
    """Parses pytype log content to count errors.

    Args:
        text: The raw string content of the pytype log file.

    Returns:
        The number of errors found.
    """
    m = re.search(r"(\d+)\s+errors?", text, re.I)
    return int(m.group(1)) if m else 0


def _parse_pydocstyle(text: str) -> int:
    """Parses pydocstyle log content to count violations.

    It looks for occurrences of pydocstyle error codes (e.g., 'D203').

    Args:
        text: The raw string content of the pydocstyle log file.

    Returns:
        The number of docstring violations found.
    """
    return len(re.findall(r"\bD\d{3}\b", text))


def _parse_radon(text: str) -> str:
    """Parses a radon cyclomatic complexity report to find the worst grade.

    It searches for grades in reverse order (F, E, D, ...) to find the
    highest (worst) complexity grade present in the report.

    Args:
        text: The raw string content of the radon CC report.

    Returns:
        The worst complexity grade found (from 'A' to 'F'), defaulting to 'A'.
    """
    for g in "FEDCBA":
        if re.search(rf"\b{g}\b", text):
            return g
    return "A"


class LintArtifactPage(StandardArtifactPage):
    """Generates the documentation page for linting artifacts."""

    out_md = Path("artifacts/lint.md")

    def title(self) -> str:
        """Returns the main title for the lint artifacts page."""
        return "Lint Artifacts"

    def intro(self) -> str:
        """Returns the introductory text for the lint artifacts page."""
        return (
            "Static analysis reports that keep style, formatting, typing and complexity in check.\n\n"
            "- **Ruff**: rules & auto-fixes; **Ruff Format**: code formatter\n"
            "- **codespell**: common typos\n"
            "- **mypy/pyright/pytype**: static typing at different speeds/strictness\n"
            "- **pydocstyle**: Google-style docstrings\n"
            "- **radon**: cyclomatic complexity (CC), Maintainability Index (MI)\n"
        )

    def iter_items(self) -> Iterable[tuple[str, Path]]:
        """Yields the lint artifact files to be documented, in a defined order."""
        items: list[tuple[str, Path]] = [(name, LINT_DIR / name) for name in ORDER]
        if LINT_DIR.is_dir():
            seen = set(ORDER)
            extras = sorted(
                p for p in LINT_DIR.glob("*") if p.is_file() and p.name not in seen)
            items.extend((p.name, p) for p in extras)
        return items

    def bullet_for(self, label: str, path: Path, content: str) -> Bullet:
        """Builds a detailed summary bullet for a specific lint artifact.

        This method dynamically constructs the bullet point by:
        1.  Looking up the artifact's description from the `BLURBS` constant.
        2.  Selecting the appropriate parsing function based on the artifact's
            label (filename) to extract metrics from the log content.
        3.  Formatting these metrics into a detailed summary string.
        4.  Assigning a relevant "How to use" tip.

        Args:
            label: The filename of the artifact (e.g., "ruff.log").
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A `Bullet` object populated with the title, a summary of the
            result, and usage guidance.
        """
        desc, good_template = BLURBS.get(label, ("Report", ""))
        extra = ""

        if label == "ruff.log":
            extra = f" — issues: {_parse_ruff(content)}"
        elif label == "ruff-format.log":
            w, c = _parse_ruff_format(content)
            extra = f" — would reformat: {w}, reformatted: {c}"
        elif label == "codespell.log":
            extra = f" — typos: {_parse_codespell(content)}"
        elif label == "mypy.log":
            ok, n = _parse_mypy(content)
            extra = f" — {'success' if ok else f'errors: {n}'}"
        elif label == "pyright.log":
            e, w = _parse_pyright(content)
            extra = f" — errors: {e}, warnings: {w}"
        elif label == "pytype.log":
            if _pytype_skipped(content):
                extra = " — skipped (Python ≥ 3.13)"
            else:
                extra = f" — errors: {_parse_pytype(content)}"
        elif label == "pydocstyle.log":
            extra = f" — violations: {_parse_pydocstyle(content)}"
        elif label == "radon.log":
            extra = f" — worst CC grade: {_parse_radon(content)}"
        elif label == "_passed":
            extra = " — OK"

        title = f"[{label}](#{anchor_for(label)}) — {desc}"
        good_line = f"{good_template}{extra}"

        usage = {
            "ruff.log": "Run `ruff --fix` for auto-fixes; justify any `# noqa`.",
            "ruff-format.log": "Run `ruff format`; enforce via pre-commit.",
            "codespell.log": "Accept valid suggestions; collect custom words.",
            "mypy.log": "Annotate public APIs; enable stricter flags gradually.",
            "pyright.log": "Use strict mode per module; ratchet warnings down.",
            "pytype.log": "Prefer precise hints over `Any`; use pragmas sparingly.",
            "pydocstyle.log": "Follow Google sections (Args/Returns/Raises); set `--convention=google`.",
            "radon.log": "Refactor hotspots (C–F); track MI/CC thresholds.",
            "_passed": "All checks green.",
        }.get(label)

        return Bullet(title=title, good=good_line, usage=usage)

    def detail_for(self, label: str, path: Path, content: str) -> str:
        """Creates the detailed description for a given lint artifact.

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
        lines = [
            f"**What it is:** {desc}",
            f"**What good looks like:** {good}",
        ]
        return '!!! info "About this report & how to act on it"\n\n' + indent("\n".join(lines) + "\n")
