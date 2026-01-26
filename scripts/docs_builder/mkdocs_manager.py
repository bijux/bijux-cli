# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Main build manager for generating the MkDocs documentation site.

This script serves as the entrypoint for the `mkdocs-gen-files` plugin. It
orchestrates the entire documentation generation process, including:
- Materializing curated documentation pages from `docs/`.
- Creating index pages for all documentation sections.
- Building detailed pages for CI/CD artifacts (linting, testing, etc.).
- Composing a complete `nav.md` file for the `literate-nav` plugin to
  construct the site navigation.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import List
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
from scripts.docs_builder.helpers import NAV_FILE
from scripts.docs_builder.helpers import REPO_ROOT
from scripts.docs_builder.helpers import ensure_top_anchor
from scripts.docs_builder.helpers import final_fixups
from scripts.docs_builder.helpers import fs_read_text
from scripts.docs_builder.helpers import nav_add_bullets
from scripts.docs_builder.helpers import nav_header
from scripts.docs_builder.helpers import rewrite_links_general
from scripts.docs_builder.helpers import write_if_changed

PAGE_META_NO_EDIT = "---\nhide:\n  - edit\n---\n\n"


def _stage_root_docs() -> None:
    """Copy key project files into the docs site; create fallbacks if absent."""
    pairs: List[Tuple[Path, Path, Callable[[str], str]]] = [
        (REPO_ROOT / "docs" / "index.md", Path("index.md"), rewrite_links_general),
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
            "- [Getting Started](getting-started/index.md)\n"
            "- [Concepts](concepts/index.md)\n"
            "- [Guides](guides/index.md)\n"
            "- [Reference](reference/index.md)\n"
            "- [Examples](examples/index.md)\n"
            "- [Artifacts](artifacts/index.md)\n"
        )
        write_if_changed("index.md", fallback)


def _stage_docs_tree() -> None:
    """Copy the curated docs tree into the generated filesystem."""
    docs_root = REPO_ROOT / "docs"
    for rel_dir in (
        "getting-started",
        "concepts",
        "guides",
        "reference",
        "examples",
        "architecture",
    ):
        src_dir = docs_root / rel_dir
        if not src_dir.is_dir():
            continue
        for src in sorted(src_dir.rglob("*.md")):
            dst = Path(rel_dir) / src.relative_to(src_dir)
            raw = fs_read_text(src)
            md = ensure_top_anchor(rewrite_links_general(raw))
            md = final_fixups(md)
            md = PAGE_META_NO_EDIT + md
            write_if_changed(dst, md)


def _build_nav() -> None:
    """Programmatically composes the entire site navigation in `nav.md`.

    This function builds a Markdown list that `mkdocs-literate-nav` uses to
    create the site's navigation tree. The structure is highly ordered and
    builds several main sections, including top-level pages and artifact
    reports.
    """

    nav = nav_header()
    nav = nav_add_bullets(
        nav,
        [
            "* [Home](index.md)",
            "* [Getting Started](getting-started/index.md)",
            f"{INDENT1}* [Installation](getting-started/installation.md)",
            f"{INDENT1}* [Quickstart](getting-started/quickstart.md)",
            "* [Concepts](concepts/index.md)",
            f"{INDENT1}* [Architecture](concepts/architecture.md)",
            f"{INDENT1}* [Execution model](concepts/execution-model.md)",
            f"{INDENT1}* [Precedence](concepts/precedence.md)",
            f"{INDENT1}* [Exit policy](concepts/exit-policy.md)",
            f"{INDENT1}* [Logging](concepts/logging.md)",
            f"{INDENT1}* [Plugin lifecycle](concepts/plugin-lifecycle.md)",
            "* [Guides](guides/index.md)",
            f"{INDENT1}* [CLI usage](guides/cli-usage.md)",
            f"{INDENT1}* [Configuration](guides/configuration.md)",
            f"{INDENT1}* [Plugins](guides/plugins.md)",
            f"{INDENT1}* [API usage](guides/api-usage.md)",
            f"{INDENT1}* [Development](guides/development.md)",
            f"{INDENT1}* [Contributor mental model](guides/contributor-mental-model.md)",
            "* [Reference](reference/index.md)",
            f"{INDENT1}* [Commands](reference/commands.md)",
            f"{INDENT1}* [Config schema](reference/config-schema.md)",
            f"{INDENT1}* [Environment](reference/environment.md)",
            f"{INDENT1}* [Exit codes](reference/exit-codes.md)",
            f"{INDENT1}* [Glossary](reference/glossary.md)",
            f"{INDENT1}* [Pre-1.0 change policy](reference/pre-1.0.md)",
            "* [Examples](examples/index.md)",
            f"{INDENT1}* [Workflows](examples/workflows.md)",
            f"{INDENT1}* [Plugins](examples/plugins.md)",
            "* [Architecture](architecture/index.md)",
            f"{INDENT1}* [Decision rules](architecture/decision-rules.md)",
            f"{INDENT1}* [Walk-through](architecture/walkthrough.md)",
        ],
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
    2. Materialize the docs tree.
    3. Build all artifact-specific documentation pages.
    4. Compose the final site navigation file.
    """
    _stage_root_docs()
    _stage_docs_tree()
    TestArtifactPage().build()
    LintArtifactPage().build()
    QualityArtifactPage().build()
    SecurityArtifactPage().build()
    APIArtifactPage().build()
    SBOMArtifactPage().build()
    CitationArtifactPage().build()
    _build_nav()


if __name__ == "__main__":
    main()
