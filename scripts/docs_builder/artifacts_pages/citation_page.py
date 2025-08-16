# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Builds the documentation page for software citation artifacts.

This module defines `CitationArtifactPage`, which finds and parses
`CITATION.cff` and its generated derivatives (BibTeX, RIS, EndNote).
Unlike other artifact pages, this implementation overrides the default build
behavior to embed the full content of each citation file directly onto the
page for easy viewing and copying.

Module Constants:
    CITATION_DIR: The directory where generated citation artifacts are stored.
    ROOT_CFF: The path to the canonical `CITATION.cff` file in the project root.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Iterable

from scripts.docs_builder.artifacts_pages.base import StandardArtifactPage
from scripts.docs_builder.artifacts_pages.base import Bullet
from scripts.docs_builder.artifacts_pages.base import bullet_block
from scripts.docs_builder.helpers import anchor_for
from scripts.docs_builder.helpers import indent
from scripts.docs_builder.helpers import write_if_changed

CITATION_DIR = Path("artifacts/citation")
ROOT_CFF = Path("CITATION.cff")


def _summ_authors(authors: list[str] | None, limit: int = 2) -> str:
    """Creates a summarized string from a list of author names.

    Args:
        authors: A list of author names.
        limit: The maximum number of authors to list before adding "et al.".

    Returns:
        A formatted string, e.g., "Poe, Edgar Allan" or "Poe, Edgar Allan,
        Lovecraft, H.P. et al.".
    """
    if not authors:
        return ""
    if len(authors) <= limit:
        return ", ".join(authors)
    return f"{', '.join(authors[:limit])} et al."


def _parse_bibtex(fp: Path) -> dict:
    """Parses a BibTeX (.bib) file using regular expressions.

    This is a simple, fault-tolerant parser designed to extract common fields
    without requiring a full BibTeX parsing library.

    Args:
        fp: The path to the .bib file.

    Returns:
        A dictionary containing extracted fields like 'title', 'authors',
        'year', 'doi', 'url', 'key', and 'entry_type'. Returns an empty
        dictionary if the file cannot be read or parsed.
    """
    try:
        text = fp.read_text(encoding="utf-8")
    except OSError:
        return {}
    md: dict[str, object] = {}
    m = re.search(r"@\s*(\w+)\s*{\s*([^,\s]+)\s*,", text, re.S | re.I)
    if m:
        md["entry_type"], md["key"] = m.group(1), m.group(2)

    def grab(field: str) -> str | None:
        """Extracts the value of a given field."""
        m1 = re.search(rf"{field}\s*=\s*{{(.*?)}}", text, re.S | re.I)
        m2 = re.search(rf'{field}\s*=\s*"(.*?)"', text, re.S | re.I)
        val = m1.group(1) if m1 else (m2.group(1) if m2 else None)
        return re.sub(r"\s+", " ", val).strip() if val else None

    md["title"] = grab("title")
    md["year"] = grab("year")
    md["doi"] = grab("doi")
    md["url"] = grab("url")
    authors = grab("author")
    if authors:
        md["authors"] = [a.strip() for a in re.split(r"\s+and\s+", authors, flags=re.I) if a.strip()]
    return md


def _parse_ris(fp: Path) -> dict:
    """Parses a RIS (.ris) file.

    Args:
        fp: The path to the .ris file.

    Returns:
        A dictionary containing extracted fields like 'title', 'authors',
        'year', 'doi', and 'url'. Returns an empty dictionary if the file
        cannot be read.
    """
    try:
        lines = fp.read_text(encoding="utf-8").splitlines()
    except OSError:
        return {}
    md = {"authors": []}
    for ln in lines:
        if ln.startswith(("TI  - ", "T1  - ")):
            md["title"] = ln[6:].strip()
        elif ln.startswith("AU  - "):
            md["authors"].append(ln[6:].strip())
        elif ln.startswith("PY  - "):
            md["year"] = ln[6:].strip()[:4]
        elif ln.startswith("DO  - "):
            md["doi"] = ln[6:].strip()
        elif ln.startswith("UR  - "):
            md["url"] = ln[6:].strip()
    return md


def _parse_enw(fp: Path) -> dict:
    """Parses an EndNote (.enw) file.

    Args:
        fp: The path to the .enw file.

    Returns:
        A dictionary containing extracted fields like 'title', 'authors',
        'year', 'doi', and 'url'. Returns an empty dictionary if the file
        cannot be read.
    """
    try:
        lines = fp.read_text(encoding="utf-8").splitlines()
    except OSError:
        return {}
    md = {"authors": []}
    for ln in lines:
        if ln.startswith("%T "):
            md["title"] = ln[3:].strip()
        elif ln.startswith("%A "):
            md["authors"].append(ln[3:].strip())
        elif ln.startswith("%D "):
            md["year"] = ln[3:].strip()[:4]
        elif ln.startswith("%R "):
            md["doi"] = ln[3:].strip()
        elif ln.startswith("%U "):
            md["url"] = ln[3:].strip()
    return md


def _parse_cff(fp: Path) -> dict:
    """Parses a Citation File Format (.cff) file using regular expressions.

    Args:
        fp: The path to the CITATION.cff file.

    Returns:
        A dictionary containing extracted fields like 'title', 'version',
        'authors', 'year', 'doi', and 'url'. Returns an empty dictionary if the
        file cannot be read.
    """
    try:
        text = fp.read_text(encoding="utf-8")
    except OSError:
        return {}
    md: dict[str, object] = {}

    def ygrab(key: str) -> str | None:
        """Extracts a simple key: value pair from the YAML-like CFF."""
        m = re.search(rf"^{key}:\s*\"?(.+?)\"?\s*$", text, re.M)
        return m.group(1).strip() if m else None

    md["title"] = ygrab("title")
    md["version"] = ygrab("version")
    md["doi"] = ygrab("doi")
    md["url"] = ygrab("url")
    m_year = re.search(r"^\s*year:\s*(\d{4})\s*$", text, re.M)
    if m_year:
        md["year"] = m_year.group(1)

    names, seen = [], set()
    pat = re.compile(
        r"(?:family-names:\s*\"?([^\n\"]+)\"?.*?given-names:\s*\"?([^\n\"]+)\"?)|"
        r"(?:given-names:\s*\"?([^\n\"]+)\"?.*?family-names:\s*\"?([^\n\"]+)\"?)",
        re.S,
    )
    for m in pat.finditer(text):
        full = f"{(m.group(2) or m.group(3) or '').strip()} {(m.group(1) or m.group(4) or '').strip()}".strip()
        if full and full not in seen:
            seen.add(full)
            names.append(full)
    if names:
        md["authors"] = names
    return md


def _parse(fp: Path) -> dict:
    """Dispatches to the correct parser based on file name or extension.

    Args:
        fp: The path to the citation file.

    Returns:
        A dictionary of parsed metadata from the selected parser. Returns an
        empty dictionary if the file format is unrecognized.
    """
    if fp.name == "CITATION.cff":
        return _parse_cff(fp)
    suf = fp.suffix.lower()
    if suf == ".bib":
        return _parse_bibtex(fp)
    if suf == ".ris":
        return _parse_ris(fp)
    if suf == ".enw":
        return _parse_enw(fp)
    return {}


class CitationArtifactPage(StandardArtifactPage):
    """Generates the documentation page for software citation files."""

    out_md = Path("artifacts/citation.md")

    def title(self) -> str:
        """Returns the main title for the citation artifacts page."""
        return "Citation Artifacts"

    def intro(self) -> str:
        """Returns the introductory text for the citation artifacts page."""
        return (
            "Citation metadata in multiple formats. **`CITATION.cff`** is the canonical source "
            "used by GitHub’s “Cite this repository”.\n\n"
            "- **CFF** → validate with `cffconvert`\n"
            "- **BibTeX (`.bib`)** for LaTeX/Pandoc\n"
            "- **RIS (`.ris`)** for Zotero/Mendeley\n"
            "- **EndNote (`.enw`)** for EndNote\n"
            "\nRun `make citation` to validate the CFF and generate all formats (ephemeral venv).\n"
        )

    def iter_items(self) -> Iterable[tuple[str, Path]]:
        """Yields the citation artifacts to be documented in a prioritized order.

        The order is: `CITATION.cff` from the root, then preferred formats
        (`.bib`, `.ris`, `.enw`) from the artifacts directory, followed by any
        other files found in that directory.

        Yields:
            An iterable of (label, path) tuples for each artifact.
        """
        files: list[Path] = []
        if ROOT_CFF.exists():
            files.append(ROOT_CFF)
        if CITATION_DIR.is_dir():
            preferred = ["citation.bib", "citation.ris", "citation.enw"]
            files.extend([CITATION_DIR / n for n in preferred if (CITATION_DIR / n).exists()])
            seen = {p.name for p in files}
            files.extend(sorted(p for p in CITATION_DIR.glob("*") if p.is_file() and p.name not in seen))
        return [(p.name, p) for p in files]

    def bullet_for(self, label: str, path: Path, content: str) -> Bullet:
        """Builds a detailed summary bullet for a specific citation artifact.

        This method determines the file type, parses it for metadata, composes
        a dynamic summary line from that metadata, and provides format-specific
        "good state" and "usage" text.

        Args:
            label: The filename of the artifact.
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A `Bullet` object populated with a title, summary, and usage guidance.
        """
        md = _parse(path)
        kind = (
            "CFF (source)"
            if label == "CITATION.cff"
            else "BibTeX"
            if label.endswith(".bib")
            else "RIS"
            if label.endswith(".ris")
            else "EndNote"
            if label.endswith(".enw")
            else "Citation"
        )
        parts: list[str] = [kind]
        title = md.get("title")
        if title:
            parts.append(f"“{title}”")
        auths = _summ_authors(md.get("authors")) if isinstance(md.get("authors"), list) else ""
        if auths:
            parts.append(auths)
        if md.get("year"):
            parts.append(str(md["year"]))
        if md.get("doi"):
            parts.append(f"DOI {md['doi']}")

        if label == "CITATION.cff":
            good = "Valid CFF (cffconvert) with title/version/authors set."
            usage = "Edit this file; run `make citation` to validate & regenerate other formats."
        elif label.endswith(".bib"):
            key = md.get("key") or Path(label).stem
            good = "Present, non-empty entry; fields (title/authors/year/doi) populated."
            usage = f"LaTeX: include & `\\cite{{{key}}}`; Pandoc: `--citeproc`. Rebuild with `make citation-bibtex`."
        elif label.endswith(".ris"):
            good = "Present, non-empty RIS with standard tags (TI/AU/PY/DO/UR)."
            usage = "Zotero/Mendeley: File → Import the `.ris`."
        elif label.endswith(".enw"):
            good = "Present, non-empty EndNote tagged format."
            usage = "EndNote: File → Import the `.enw`."
        else:
            good = "Well-formed citation file."
            usage = "Use in your reference manager."

        extra = " • ".join(parts)
        title_line = f"[{label}](#{anchor_for(label)}) — {kind}"
        good_line = f"{good}" + (f" — {extra}" if extra else "")
        return Bullet(title=title_line, good=good_line, usage=usage)

    def detail_for(self, label: str, path: Path, content: str) -> str:
        """Creates the detailed description for a given citation artifact.

        This method generates a rich 'info' block that displays the metadata
        parsed from the file (title, authors, DOI, etc.). It also provides a
        set of specific usage tips and code snippets tailored to each format.

        Args:
            label: The filename of the artifact.
            path: The path to the artifact file.
            content: The raw string content of the artifact file.

        Returns:
            A formatted Markdown string for the artifact's detail section.
        """
        md = _parse(path)
        lines: list[str] = []
        if md.get("title"):
            lines.append(f"**Title:** {md['title']}")
        auths_full = _summ_authors(md.get("authors"), limit=99) if isinstance(md.get("authors"), list) else ""
        if auths_full:
            lines.append(f"**Authors:** {auths_full}")
        if md.get("year"):
            lines.append(f"**Year:** {md['year']}")
        if md.get("version"):
            lines.append(f"**Version:** {md['version']}")
        if md.get("doi"):
            lines.append(f"**DOI:** {md['doi']}")
        if md.get("url"):
            lines.append(f"**URL:** {md['url']}")
        try:
            lines.append(f"**File size:** {path.stat().st_size} bytes")
        except OSError:
            pass

        extra = ""
        if label == "CITATION.cff":
            extra = (
                "\n- **Validate:** `make citation-validate` (uses ephemeral venv with `cffconvert`).\n"
                "- **Generate all formats:** `make citation`.\n"
                "- **CI tip:** Run `make citation` and commit artifacts under `artifacts/citation/` on release."
            )
        elif label.endswith(".bib"):
            key = md.get("key") or Path(label).stem
            extra = (
                "\n**LaTeX (BibTeX):**\n"
                "```tex\n"
                "\\bibliographystyle{plain}\n"
                f"\\bibliography{{{Path(label).stem}}}\n"
                "\n"
                f"In text: \\cite{{{key}}}\n"
                "```\n"
                "\n"
                "**Pandoc:**\n"
                "```bash\n"
                f"pandoc paper.md --citeproc --bibliography {label} -o paper.pdf\n"
                "```"
            )
        elif label.endswith(".ris"):
            extra = "\n- **Reference managers:** Import this `.ris` into Zotero/Mendeley."
        elif label.endswith(".enw"):
            extra = "\n- **Reference manager:** Import this `.enw` into EndNote."

        joined = "\n".join(lines) + extra + "\n"
        return '!!! info "About this citation & how to use it"\n\n' + indent(joined)

    def _lang_for_label(self, label: str) -> str:
        """Determines the Markdown language identifier for a given artifact label.

        This is used to apply syntax highlighting to the embedded code blocks.

        Args:
            label: The filename of the artifact.

        Returns:
            A string representing the language for a fenced code block, e.g.,
            'yaml', 'bibtex', or 'text'.
        """
        n = label.lower()
        if n.endswith(".cff"):
            return "yaml"
        if n.endswith(".bib"):
            return "bibtex"
        if n.endswith(".ris") or n.endswith(".enw"):
            return "text"
        return "text"

    def build(self) -> None:
        """Generates the complete Markdown page with inline artifact content.

        This method overrides the base class's build logic. Instead of writing
        artifact content to separate sidecar files, it embeds the full, raw
        content of each citation file directly within a syntax-highlighted code
        block on the main page. This is done because citation files are typically
        small and benefit from being immediately visible.

        The overall page structure (title, intro, bullets, details) is preserved.
        """
        page: list[str] = [f"# {self.title()} {{#top}}\n\n"]
        page.append('!!! tip "What are these files?"\n\n')
        page.append(indent(self.intro()))
        page.append("\n")

        items = [(lbl, p) for (lbl, p) in self.iter_items() if p.exists()]
        if not items:
            page.append("_No files found._\n")
            write_if_changed(self.out_md, "".join(page))
            return

        bullets_content: list[str] = []
        details_content: list[str] = []

        for label, path in items:
            content = self._read(path)

            b = self.bullet_for(label, path, content)
            bullets_content.append(bullet_block(b.title, b.good, b.usage))

            section_md: list[str] = []
            section_md.append(f"## {label} {{#{anchor_for(label)}}}\n\n")
            about = self.detail_for(label, path, content)
            if about:
                section_md.append(about)
                section_md.append("\n")

            if path.is_file():
                lang = self._lang_for_label(label)
                section_md.append(f"```{lang}\n{content}\n```\n\n")

            details_content.append("".join(section_md))

        page.append("**Files & what they’re for**\n\n")
        page.extend(bullets_content)
        page.append("\n---\n\n")
        page.extend(details_content)

        write_if_changed(self.out_md, "".join(page))
