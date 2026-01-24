# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Main build manager for generating the MkDocs documentation site.

This script serves as the entrypoint for the `mkdocs-gen-files` plugin. It
orchestrates the entire documentation generation process, including:
- Materializing top-level project Markdown files (e.g., README, USAGE).
- Materializing focused guides (plugins, examples).
- Finding and processing Architecture Decision Records (ADRs).
- Creating index pages for all documentation sections.
- Building detailed pages for CI/CD artifacts (linting, testing, etc.).
- Composing a complete `nav.md` file for the `literate-nav` plugin to
  construct the site navigation.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import List
from typing import Optional
from typing import Tuple
from typing import Callable

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.docs_builder.artifacts_pages.api_page import APIArtifactPage
from scripts.docs_builder.artifacts_pages.citation_page import CitationArtifactPage
from scripts.docs_builder.artifacts_pages.lint_page import LintArtifactPage
from scripts.docs_builder.artifacts_pages.quality_page import QualityArtifactPage
from scripts.docs_builder.artifacts_pages.sbom_page import SBOMArtifactPage
from scripts.docs_builder.artifacts_pages.security_page import SecurityArtifactPage
from scripts.docs_builder.artifacts_pages.test_page import TestArtifactPage
from scripts.docs_builder.helpers import INDENT1
from scripts.docs_builder.helpers import INDENT2
from scripts.docs_builder.helpers import NAV_FILE
from scripts.docs_builder.helpers import REPO_ROOT
from scripts.docs_builder.helpers import ensure_top_anchor
from scripts.docs_builder.helpers import final_fixups
from scripts.docs_builder.helpers import fs_read_text
from scripts.docs_builder.helpers import nav_add_bullets
from scripts.docs_builder.helpers import nav_header
from scripts.docs_builder.helpers import rewrite_links_general
from scripts.docs_builder.helpers import rewrite_links_tree
from scripts.docs_builder.helpers import write_if_changed

ADR_SRC_PRIMARY = REPO_ROOT / "ADR"
ADR_SRC_FALLBACK = REPO_ROOT / "docs" / "ADR"
ADR_DEST_DIR = Path("ADR")

PAGE_META_NO_EDIT = "---\nhide:\n  - edit\n---\n\n"


def _pick_adr_source() -> Optional[Path]:
    """Selects the source directory for Architecture Decision Records (ADRs).

    It prefers the top-level `ADR/` directory. If that does not exist, it
    falls back to `docs/ADR/`.

    Returns:
        The path to the ADR source directory, or None if neither exists.
    """
    if ADR_SRC_PRIMARY.is_dir():
        return ADR_SRC_PRIMARY
    if ADR_SRC_FALLBACK.is_dir():
        return ADR_SRC_FALLBACK
    return None


def _iter_adr_files(src_root: Path) -> List[Path]:
    """Lists all ADR Markdown files in a directory, sorted by name.

    It excludes any `index.md` file from the list.

    Args:
        src_root: The directory to search for ADR files.

    Returns:
        A sorted list of paths to the ADR files.
    """
    return sorted(
        [p for p in src_root.glob("*.md") if p.is_file() and p.name != "index.md"],
        key=lambda p: p.name,
    )


def _adr_display_name(filename: str) -> str:
    """Formats a user-friendly title from an ADR filename.

    For example, "0001-some-decision.md" becomes "ADR 0001: Some Decision".

    Args:
        filename: The name of the ADR file.

    Returns:
        A formatted, human-readable title string.
    """
    stem = filename[:-3]
    parts = stem.split("-", 1)
    if len(parts) == 2 and parts[0].isdigit():
        adr_num, title_raw = parts
        return f"ADR {adr_num.zfill(4)}: {title_raw.replace('-', ' ').title()}"
    return stem.replace("-", " ").title()


def _materialize_root_docs() -> None:
    """Copy key project files into the docs site; create fallbacks if absent."""
    pairs: List[Tuple[Path, Path, Callable[[str], str]]] = [
        (REPO_ROOT / "README.md", Path("index.md"), rewrite_links_general),
        (REPO_ROOT / "docs" / "USAGE.md", Path("usage.md"), rewrite_links_general),
        (REPO_ROOT / "docs" / "examples.md", Path("examples.md"), rewrite_links_general),
        (
            REPO_ROOT / "docs" / "plugins" / "index.md",
            Path("plugins/index.md"),
            rewrite_links_general,
        ),
        (
            REPO_ROOT / "docs" / "plugins" / "lifecycle.md",
            Path("plugins/lifecycle.md"),
            rewrite_links_general,
        ),
        (REPO_ROOT / "docs" / "TESTS.md", Path("tests.md"), rewrite_links_general),
        (
            REPO_ROOT / "docs" / "PROJECT_TREE.md",
            Path("project_tree.md"),
            rewrite_links_tree,
        ),
        (REPO_ROOT / "docs" / "TOOLING.md", Path("tooling.md"), rewrite_links_general),
        (REPO_ROOT / "SECURITY.md", Path("security.md"), rewrite_links_general),
        (
            REPO_ROOT / "CODE_OF_CONDUCT.md",
            Path("code_of_conduct.md"),
            rewrite_links_general,
        ),
        (REPO_ROOT / "CONTRIBUTING.md", Path("contributing.md"), rewrite_links_general),
        (REPO_ROOT / "CHANGELOG.md", Path("changelog.md"), rewrite_links_general),
        (
            REPO_ROOT / "LICENSES" / "Apache-2.0.txt",
            Path("license.md"),
            rewrite_links_general,
        ),
    ]
    have_index = False
    for src, dst, fixer in pairs:
        if not src.exists():
            continue
        raw = fs_read_text(src)
        md = ensure_top_anchor(fixer(raw))
        md = final_fixups(md)
        md = PAGE_META_NO_EDIT + md
        write_if_changed(dst, md)
        if dst.as_posix() == "index.md":
            have_index = True

    if not have_index:
        fallback = PAGE_META_NO_EDIT + (
            "# Bijux CLI {#top}\n\n"
            "_Auto-generated skeleton page._\n\n"
            "- [Usage](usage.md)\n"
            "- [Plugins](plugins/index.md)\n"
            "- [Examples](examples.md)\n"
            "- [Artifacts](artifacts/index.md)\n"
            "- [Architecture Decision Records](ADR/index.md)\n"
        )
        write_if_changed("index.md", fallback)


def _materialize_adrs() -> None:
    """Copies ADRs from the source directory into the virtual docs filesystem.

    This step is skipped if the ADRs are already located in the on-disk
    `docs/ADR/` directory, as `mkdocs-gen-files` will pick them up automatically.
    """
    src_root = _pick_adr_source()
    if not src_root or src_root == ADR_SRC_FALLBACK:
        return

    for src in _iter_adr_files(src_root):
        dst = ADR_DEST_DIR / src.name
        raw = fs_read_text(src)
        md = ensure_top_anchor(rewrite_links_general(raw))
        md = final_fixups(md)
        md = PAGE_META_NO_EDIT + md
        write_if_changed(dst, md)


def _generate_adr_index() -> None:
    """Generates the `ADR/index.md` file in the virtual docs filesystem.

    This ensures a correct and up-to-date index is always available,
    regardless of whether an index file exists in the source directory.
    """
    src_root = _pick_adr_source()
    if not src_root:
        return
    files = _iter_adr_files(src_root)
    if not files:
        return

    lines = [PAGE_META_NO_EDIT, "# Architecture Decision Records {#top}\n\n"]
    for p in files:
        lines.append(f"- [{_adr_display_name(p.name)}](./{p.name})\n")
    write_if_changed(ADR_DEST_DIR / "index.md", "".join(lines))


def _compose_nav() -> None:
    """Programmatically composes the entire site navigation in `nav.md`.

    This function builds a Markdown list that `mkdocs-literate-nav` uses to
    create the site's navigation tree. The structure is highly ordered and
    builds several main sections, including top-level pages, a nested API
    Reference section, ADRs, and artifact reports.
    """

    nav = nav_header()
    nav = nav_add_bullets(
        nav,
        [
            "* [Home](index.md)",
            "* [Usage](usage.md)",
            "* [Plugins](plugins/index.md)",
            f"{INDENT1}* [Lifecycle](plugins/lifecycle.md)",
            "* [Examples](examples.md)",
            "* [Project Overview](project_tree.md)",
            "* [Tests](tests.md)",
            "* [Tooling](tooling.md)",
        ],
    )

    src_root = _pick_adr_source()
    if src_root and (files := _iter_adr_files(src_root)):
        nav = nav_add_bullets(
            nav, ["* Architecture", f"{INDENT1}* [Decision Records](ADR/index.md)"]
        )
        for p in files:
            nav = nav_add_bullets(
                nav, [f"{INDENT2}* [{_adr_display_name(p.name)}](ADR/{p.name})"]
            )

    nav = nav_add_bullets(nav, ["* [Changelog](changelog.md)"])

    community_pages = [
        ("Code of Conduct", "code_of_conduct.md"),
        ("Contributing", "contributing.md"),
        ("Security Policy", "security.md"),
        ("License", "license.md"),
    ]
    landing = [
        PAGE_META_NO_EDIT,
        "# Community {#top}\n\n",
        "Project policies and how to get involved.\n\n",
    ]
    for title, path in community_pages:
        landing.append(f"- [{title}]({path})\n")
    write_if_changed(Path("community.md"), "".join(landing))
    nav = nav_add_bullets(nav, ["* [Community](community.md)"])
    for title, path in community_pages:
        nav = nav_add_bullets(nav, [f"{INDENT1}* [{title}]({path})"])

    artifacts = [
        ("Test Artifacts", "artifacts/test.md"),
        ("Lint Artifacts", "artifacts/lint.md"),
        ("Quality Artifacts", "artifacts/quality.md"),
        ("Security Artifacts", "artifacts/security.md"),
        ("API Artifacts", "artifacts/api.md"),
        ("SBOM Artifacts", "artifacts/sbom.md"),
        ("Citation Artifacts", "artifacts/citation.md"),
    ]
    landing = [
        PAGE_META_NO_EDIT,
        "# Artifacts {#top}\n\n",
        "Collected CI/test reports and logs.\n\n",
    ]
    for title, path in artifacts:
        landing.append(f"- [{title}]({Path(path).name})\n")
    write_if_changed(Path("artifacts/index.md"), "".join(landing))

    nav = nav_add_bullets(nav, ["* [Artifacts](artifacts/index.md)"])
    for title, path in artifacts:
        nav = nav_add_bullets(nav, [f"{INDENT1}* [{title}]({path})"])

    write_if_changed(NAV_FILE, nav)


def main() -> None:
    """The main entrypoint for the documentation generation script.

    Orchestrates the entire build process by calling functions in sequence to:
    1. Materialize root documentation files.
    2. Materialize and index ADRs.
    3. Generate API reference pages and their indexes.
    4. Build all artifact-specific documentation pages.
    5. Compose the final site navigation file.
    """
    _materialize_root_docs()
    _materialize_adrs()
    _generate_adr_index()
    TestArtifactPage().build()
    LintArtifactPage().build()
    QualityArtifactPage().build()
    SecurityArtifactPage().build()
    APIArtifactPage().build()
    SBOMArtifactPage().build()
    CitationArtifactPage().build()
    _compose_nav()


if __name__ == "__main__":
    main()
