# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Main build manager for generating the MkDocs documentation site.

This script serves as the entrypoint for the `mkdocs-gen-files` plugin. It
orchestrates the entire documentation generation process, including:
- Materializing top-level project Markdown files (e.g., README, USAGE).
- Finding and processing Architecture Decision Records (ADRs).
- Generating API reference documentation from Python source files using
  `mkdocstrings`.
- Creating index pages for all documentation sections.
- Building detailed pages for CI/CD artifacts (linting, testing, etc.).
- Composing a complete `nav.md` file for the `literate-nav` plugin to
  construct the site navigation.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Dict
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
from scripts.docs_builder.helpers import INDENT3
from scripts.docs_builder.helpers import NAV_FILE
from scripts.docs_builder.helpers import REPO_ROOT
from scripts.docs_builder.helpers import SRC_DIR
from scripts.docs_builder.helpers import ensure_top_anchor
from scripts.docs_builder.helpers import final_fixups
from scripts.docs_builder.helpers import fs_read_text
from scripts.docs_builder.helpers import nav_add_bullets
from scripts.docs_builder.helpers import nav_header
from scripts.docs_builder.helpers import pretty_title
from scripts.docs_builder.helpers import rewrite_links_general
from scripts.docs_builder.helpers import rewrite_links_tree
from scripts.docs_builder.helpers import write_if_changed

ADR_SRC_PRIMARY = REPO_ROOT / "ADR"
ADR_SRC_FALLBACK = REPO_ROOT / "docs" / "ADR"
ADR_DEST_DIR = Path("ADR")

PAGE_META_NO_EDIT = (
    "---\nhide:\n  - edit\n---\n\n"
)

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
        (REPO_ROOT / "README.md",         Path("index.md"),        rewrite_links_general),
        (REPO_ROOT / "USAGE.md",          Path("usage.md"),        rewrite_links_general),
        (REPO_ROOT / "TESTS.md",          Path("tests.md"),        rewrite_links_general),
        (REPO_ROOT / "PROJECT_TREE.md",   Path("project_tree.md"), rewrite_links_tree),
        (REPO_ROOT / "TOOLING.md",        Path("tooling.md"),      rewrite_links_general),
        (REPO_ROOT / "SECURITY.md",       Path("security.md"),     rewrite_links_general),
        (REPO_ROOT / "CODE_OF_CONDUCT.md",Path("code_of_conduct.md"), rewrite_links_general),
        (REPO_ROOT / "CONTRIBUTING.md",   Path("contributing.md"), rewrite_links_general),
        (REPO_ROOT / "CHANGELOG.md",      Path("changelog.md"),    rewrite_links_general),
        (REPO_ROOT / "LICENSES" / "MIT.txt", Path("license.md"),   rewrite_links_general),
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
            "- [API Reference](reference/index.md)\n"
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


def _generate_api_pages() -> Dict[str, List[Tuple[str, str]]]:
    """Walks the source directory and generates API reference pages.

    For each Python module found in the `SRC_DIR`, this function creates a
    corresponding Markdown file in the `reference/` virtual directory. The
    content of each file is a `mkdocstrings` block configured to render the
    API documentation for that module.

    Returns:
        A dictionary mapping each reference subdirectory to a list of pages
        it contains. Each page is a tuple of (display_name, path). This
        structure is used to build index pages and the site navigation.
    """
    ref_dir_to_pages: Dict[str, List[Tuple[str, str]]] = {}
    for root, _, files in os.walk(SRC_DIR):
        rel_root = os.path.relpath(root, SRC_DIR)
        section = None if rel_root == "." else rel_root
        for file in files:
            if not file.endswith(".py") or file.startswith("__") or file == "py.typed":
                continue

            module_name = file[:-3]
            raw_md_path = os.path.join("reference", rel_root, f"{module_name}.md")
            md_path = os.path.normpath(raw_md_path).replace("\\", "/")
            is_command = (section or "").split(os.sep, 1)[0] == "commands"

            header = f"# {module_name.capitalize()} Command API Reference\n" if is_command else f"# {module_name.capitalize()} Module API Reference\n"
            blurb = f"This section documents the internals of the `{module_name}` command in Bijux CLI.\n" if is_command else f"This section documents the internals of the `{module_name}` module in Bijux CLI.\n"
            full_module_path = f"bijux_cli.{module_name}" if section is None else f"bijux_cli.{section.replace(os.sep, '.')}.{module_name}"
            content = (
                PAGE_META_NO_EDIT
                + header
                + blurb
                + f"::: {full_module_path}\n"
                + "    handler: python\n"
                + "    options:\n"
                + "      show_root_heading: true\n"
                + "      show_source: true\n"
                + "      show_signature_annotations: true\n"
                + "      docstring_style: google\n"
            )
            write_if_changed(Path(md_path), content)

            label = "Command" if is_command else "Module"
            display_name = f"{pretty_title(Path(md_path).stem)} {label}"
            ref_dir = os.path.dirname(md_path) or "reference"
            ref_dir_to_pages.setdefault(ref_dir, []).append((display_name, md_path))
    return ref_dir_to_pages


def _write_reference_indexes(ref_dir_to_pages: Dict[str, List[Tuple[str, str]]]) -> set[str]:
    """Creates `index.md` files for all API reference directories.

    Args:
        ref_dir_to_pages: The mapping of directories to pages from `_generate_api_pages`.

    Returns:
        A set of all directory paths within the API reference section.
    """
    all_dirs: set[str] = {"reference"}
    for ref_dir in ref_dir_to_pages:
        parts = ref_dir.split("/")
        for i in range(1, len(parts) + 1):
            all_dirs.add("/".join(parts[:i]))

    for ref_dir in sorted(all_dirs):
        title = ref_dir.replace("reference", "Reference").strip("/").replace("/", " / ") or "Reference"
        lines = [PAGE_META_NO_EDIT, f"# {title.title()} Index\n\n"]
        for display_name, md_link in sorted(ref_dir_to_pages.get(ref_dir, []), key=lambda x: x[0].lower()):
            lines.append(f"- [{display_name}]({Path(md_link).name})\n")
        write_if_changed(Path(ref_dir) / "index.md", "".join(lines))
    return all_dirs


def _compose_nav(ref_dir_to_pages: Dict[str, List[Tuple[str, str]]], all_dirs: set[str]) -> None:
    """Programmatically composes the entire site navigation in `nav.md`.

    This function builds a Markdown list that `mkdocs-literate-nav` uses to
    create the site's navigation tree. The structure is highly ordered and
    builds several main sections, including top-level pages, a nested API
    Reference section, ADRs, and artifact reports.

    Args:
        ref_dir_to_pages: The mapping of reference directories to pages.
        all_dirs: A set of all reference directories that exist.
    """
    nav = nav_header()
    nav = nav_add_bullets(
        nav,
        [
            "* [Home](index.md)",
            "* [Usage](usage.md)",
            "* [Project Overview](project_tree.md)",
            "* [Tests](tests.md)",
            "* [Tooling](tooling.md)",
            "* API Reference",
            f"{INDENT1}* [Index](reference/index.md)",
        ],
    )

    root_pages = ref_dir_to_pages.get("reference", [])
    root_by_stem = {Path(p).stem.lower(): (name, p) for name, p in root_pages}
    for stem in ("api", "cli", "httpapi"):
        if stem in root_by_stem:
            name, p = root_by_stem.pop(stem)
            nav = nav_add_bullets(nav, [f"{INDENT1}* [{name}]({p})"])
    for name, p in sorted(root_by_stem.values(), key=lambda x: x[0].lower()):
        nav = nav_add_bullets(nav, [f"{INDENT1}* [{name}]({p})"])

    SECTION_ORDER = ("commands", "contracts", "core", "infra", "services")
    section_dirs = [f"reference/{s}" for s in SECTION_ORDER if f"reference/{s}" in all_dirs]

    for section_dir in section_dirs:
        section_name = section_dir.split("/", 1)[1].capitalize()
        nav = nav_add_bullets(nav, [f"{INDENT1}* {section_name}", f"{INDENT2}* [Index]({section_dir}/index.md)"])
        pages_here = sorted(ref_dir_to_pages.get(section_dir, []), key=lambda x: x[0].lower())
        if pages_here:
            bucket = "Commands" if section_dir.endswith("/commands") else "Modules"
            nav = nav_add_bullets(nav, [f"{INDENT2}* {bucket}"])
            for display_name, md_link in pages_here:
                nav = nav_add_bullets(nav, [f"{INDENT3}* [{display_name}]({md_link})"])

        subdirs = sorted(d for d in all_dirs if d.startswith(section_dir + "/"))
        seen = {section_dir}
        for sub_dir in subdirs:
            if sub_dir in seen:
                continue
            seen.add(sub_dir)
            subgroup_title = pretty_title(Path(sub_dir).name)
            nav = nav_add_bullets(nav, [f"{INDENT2}* {subgroup_title}", f"{INDENT3}* [Index]({sub_dir}/index.md)"])
            for display_name, md_link in sorted(ref_dir_to_pages.get(sub_dir, []), key=lambda x: x[0].lower()):
                nav = nav_add_bullets(nav, [f"{INDENT3}* [{display_name}]({md_link})"])

    src_root = _pick_adr_source()
    if src_root and (files := _iter_adr_files(src_root)):
        nav = nav_add_bullets(nav, ["* Architecture", f"{INDENT1}* [Decision Records](ADR/index.md)"])
        for p in files:
            nav = nav_add_bullets(nav, [f"{INDENT2}* [{_adr_display_name(p.name)}](ADR/{p.name})"])

    nav = nav_add_bullets(nav, ["* [Changelog](changelog.md)"])

    community_pages = [
        ("Code of Conduct", "code_of_conduct.md"),
        ("Contributing", "contributing.md"),
        ("Security Policy", "security.md"),
        ("License", "license.md"),
    ]
    landing = [PAGE_META_NO_EDIT, "# Community {#top}\n\n",
               "Project policies and how to get involved.\n\n"]
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
        ("Citation Artifacts", "artifacts/citation.md")
    ]
    landing = [PAGE_META_NO_EDIT, "# Artifacts {#top}\n\n",
                    "Collected CI/test reports and logs.\n\n"]
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
    ref = _generate_api_pages()
    print(f"[docs] generated {sum(len(v) for v in ref.values())} reference pages")
    dirs = _write_reference_indexes(ref)
    _generate_adr_index()
    TestArtifactPage().build()
    LintArtifactPage().build()
    QualityArtifactPage().build()
    SecurityArtifactPage().build()
    APIArtifactPage().build()
    SBOMArtifactPage().build()
    CitationArtifactPage().build()
    _compose_nav(ref, dirs)


if __name__ == "__main__":
    main()
