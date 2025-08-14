# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""
MkDocs build helper: materializes top-level docs, generates API pages, and writes site navigation.

Runs under `mkdocs-gen-files` and performs:
1) Copy + rewrite root Markdown (fix links, ensure {#top} anchors):
   - README.md        -> docs/index.md
   - USAGE.md         -> docs/usage.md
   - TESTS.md         -> docs/tests.md
   - PROJECT_TREE.md  -> docs/project_tree.md
   - TOOLING.md       -> docs/tooling.md
2) Generate mkdocstrings-ready API reference pages for all modules under `src/bijux_cli/**`.
3) Create per-package reference indexes: `docs/reference/**/index.md`.
4) Produce `docs/nav.md` with: Home, User Guide, Project Tree, Tests, Tooling, API Reference, Changelog, ADRs, Community.
"""

from __future__ import annotations

import os
import re
from pathlib import Path

import mkdocs_gen_files

REPO_ROOT = Path(__file__).resolve().parent.parent
SRC_DIR = Path("src/bijux_cli")
ADR_DIR = Path("docs/ADR")
NAV_FILE = Path("nav.md")

README_PATH = REPO_ROOT / "README.md"
USAGE_PATH = REPO_ROOT / "USAGE.md"
TESTS_PATH = REPO_ROOT / "TESTS.md"
TREE_PATH = REPO_ROOT / "PROJECT_TREE.md"
TOOLING_PATH = REPO_ROOT / "TOOLING.md"

INDENT_LEVEL_1 = "    "
INDENT_LEVEL_2 = INDENT_LEVEL_1 * 2
INDENT_LEVEL_3 = INDENT_LEVEL_1 * 3
INDENT_LEVEL_4 = INDENT_LEVEL_1 * 4


def read_text(path: Path) -> str:
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def write_if_changed(rel_path: Path, content: str) -> None:
    try:
        with mkdocs_gen_files.open(str(rel_path), "r") as f:
            existing = f.read()
    except FileNotFoundError:
        existing = None
    if existing != content:
        with mkdocs_gen_files.open(str(rel_path), "w") as f:
            f.write(content)


_link_pat = re.compile(r"\]\(([^)]+)\)")


def _rewrite_links_general(md: str) -> str:
    replacements = {
        "TESTS.md": "tests.md",
        "./TESTS.md": "tests.md",
        "PROJECT_TREE.md": "project_tree.md",
        "./PROJECT_TREE.md": "project_tree.md",
        "TOOLING.md": "tooling.md",
        "./TOOLING.md": "tooling.md",
        "docs/index.md": "index.md",
        "./docs/index.md": "index.md",
    }

    def repl(m: re.Match) -> str:
        target = m.group(1)
        new = replacements.get(target)
        if new:
            return f"]({new})"
        return m.group(0)

    return _link_pat.sub(repl, md)


def _rewrite_links_tree(md: str) -> str:
    md = _rewrite_links_general(md)

    def repl(m: re.Match) -> str:
        href = m.group(1)
        if href.startswith("src/bijux_cli/") and href.endswith(".py"):
            rel = href[len("src/bijux_cli/") : -3]
            ref = "reference/" + rel + ".md"
            ref = ref.replace("\\", "/")
            return f"]({ref})"
        if href.rstrip("/").endswith("src/bijux_cli/commands"):
            return "](reference/commands/index.md)"
        if href in ("#source-code-srcbijux_cli", "#plugin-template-plugin_template"):
            return "](#top)"
        return m.group(0)

    md = _link_pat.sub(repl, md)
    md = md.replace("src/bijux_cli/cli.py", "reference/cli.md")
    md = md.replace("src/bijux_cli/commands/", "reference/commands/index.md")
    return md


def _ensure_top_anchor(md: str) -> str:
    if "{#top}" in md or 'id="top"' in md or "(#top)" in md:
        return md
    lines = md.splitlines()
    for i, line in enumerate(lines[:20]):
        if line.startswith("# "):
            lines[i] = line.rstrip() + " {#top}"
            return "\n".join(lines)
    return '<a id="top"></a>\n\n' + md


def _pretty_title(stem: str) -> str:
    return stem.replace("_", " ").title()


if README_PATH.exists():
    readme = read_text(README_PATH)
    readme = _rewrite_links_general(readme)
    readme = _ensure_top_anchor(readme)
    write_if_changed(Path("index.md"), readme)

if USAGE_PATH.exists():
    usage = read_text(USAGE_PATH)
    usage = _rewrite_links_general(usage)
    usage = _ensure_top_anchor(usage)
    write_if_changed(Path("usage.md"), usage)

if TESTS_PATH.exists():
    tests = read_text(TESTS_PATH)
    tests = _rewrite_links_general(tests)
    tests = _ensure_top_anchor(tests)
    write_if_changed(Path("tests.md"), tests)

if TREE_PATH.exists():
    tree = read_text(TREE_PATH)
    tree = _rewrite_links_tree(tree)
    tree = _ensure_top_anchor(tree)
    write_if_changed(Path("project_tree.md"), tree)

if TOOLING_PATH.exists():
    tooling = read_text(TOOLING_PATH)
    tooling = _rewrite_links_general(tooling)
    tooling = _ensure_top_anchor(tooling)
    write_if_changed(Path("tooling.md"), tooling)

nav_content = "# Full Navigation\n"
nav_content += "* [Home](index.md)\n"
nav_content += "* [User Guide](usage.md)\n"
nav_content += "* [Project Tree](project_tree.md)\n"
nav_content += "* [Tests](tests.md)\n"
nav_content += "* [Tooling](tooling.md)\n"

ref_dir_to_pages: dict[str, list[tuple[str, str]]] = {}
all_dirs: set[str] = set(["reference"])

for root, _, files in os.walk(SRC_DIR):
    rel_root = os.path.relpath(root, SRC_DIR)
    section = rel_root if rel_root != "." else None
    for file in files:
        if not file.endswith(".py"):
            continue
        if file.startswith("__") or file == "py.typed":
            continue
        module_name = file[:-3]
        raw_md_path = os.path.join("reference", rel_root, f"{module_name}.md")
        md_path = os.path.normpath(raw_md_path).replace("\\", "/")
        header = (
            f"# {module_name.capitalize()} Command API Reference\n"
            if "commands" in (section or "")
            else f"# {module_name.capitalize()} Module API Reference\n"
        )
        blurb = (
            "This section documents the internals of the "
            f"`{module_name}` command in Bijux CLI, including all "
            "arguments, options, and output structure.\n"
            if "commands" in (section or "")
            else "This section documents the internals of the "
            f"`{module_name}` module in Bijux CLI.\n"
        )
        full_module_path = (
            f"bijux_cli.{module_name}"
            if section is None
            else f"bijux_cli.{section.replace(os.sep, '.')}.{module_name}"
        )
        content = (
            header
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
        label = "Command" if (section or "").split(os.sep, 1)[0] == "commands" else "Module"
        display_name = f"{_pretty_title(Path(md_path).stem)} {label}"
        ref_dir = os.path.dirname(md_path) or "reference"
        ref_dir_to_pages.setdefault(ref_dir, []).append((display_name, md_path))
        all_dirs.add(ref_dir)

dir_children: dict[str, list[str]] = {}
for d in all_dirs:
    parent = d.rsplit("/", 1)[0] if "/" in d else ""
    dir_children.setdefault(parent, []).append(d)
for v in dir_children.values():
    v.sort()

for ref_dir in sorted(all_dirs):
    title = ref_dir.replace("reference", "Reference").strip("/").replace("/", " / ")
    if not title:
        title = "Reference"
    lines = [f"# {title.title()} Index\n\n"]
    for display_name, md_link in sorted(ref_dir_to_pages.get(ref_dir, [])):
        lines.append(f"- [{display_name}]({os.path.basename(md_link)})\n")
    write_if_changed(Path(ref_dir) / "index.md", "".join(lines))

nav_content += "* API Reference\n"

root_pages = ref_dir_to_pages.get("reference", [])
root_by_stem = {Path(p).stem.lower(): (name, p) for name, p in root_pages}
for stem in ["api", "cli", "httpapi"]:
    if stem in root_by_stem:
        name, p = root_by_stem.pop(stem)
        nav_content += f"{INDENT_LEVEL_1}* [{name}]({p})\n"
for name, p in sorted(root_by_stem.values(), key=lambda x: x[0].lower()):
    nav_content += f"{INDENT_LEVEL_1}* [{name}]({p})\n"

SECTION_ORDER = ("commands", "contracts", "core", "infra", "services")
section_dirs = [f"reference/{s}" for s in SECTION_ORDER if f"reference/{s}" in all_dirs]

for section_dir in section_dirs:
    section_name = section_dir.split("/", 1)[1].capitalize()
    nav_content += f"{INDENT_LEVEL_1}* {section_name}\n"
    nav_content += f"{INDENT_LEVEL_2}* [Index]({section_dir}/index.md)\n"
    pages_here = sorted(ref_dir_to_pages.get(section_dir, []), key=lambda x: x[0].lower())
    if pages_here:
        bucket = "Commands" if section_dir.endswith("/commands") else "Modules"
        nav_content += f"{INDENT_LEVEL_2}* {bucket}\n"
        for display_name, md_link in pages_here:
            nav_content += f"{INDENT_LEVEL_3}* [{display_name}]({md_link})\n"
    for sub_dir in sorted(d for d in dir_children.get(section_dir, []) if d != section_dir):
        subgroup_title = _pretty_title(Path(sub_dir).name)
        nav_content += f"{INDENT_LEVEL_2}* {subgroup_title}\n"
        nav_content += f"{INDENT_LEVEL_3}* [Index]({sub_dir}/index.md)\n"
        for display_name, md_link in sorted(ref_dir_to_pages.get(sub_dir, []), key=lambda x: x[0].lower()):
            nav_content += f"{INDENT_LEVEL_3}* [{display_name}]({md_link})\n"
        for sub_sub in sorted(d for d in dir_children.get(sub_dir, []) if d != sub_dir):
            title = _pretty_title(Path(sub_sub).name)
            nav_content += f"{INDENT_LEVEL_3}* {title}\n"
            nav_content += f"{INDENT_LEVEL_4}* [Index]({sub_sub}/index.md)\n"
            for display_name, md_link in sorted(ref_dir_to_pages.get(sub_sub, []), key=lambda x: x[0].lower()):
                nav_content += f"{INDENT_LEVEL_4}* [{display_name}]({md_link})\n"

nav_content += "* [Changelog](changelog.md)\n"
nav_content += "* [Architecture Decision Records](ADR/index.md)\n"
if os.path.isdir(ADR_DIR):
    for file in sorted(os.listdir(ADR_DIR)):
        if not file.endswith(".md") or file == "index.md":
            continue
        parts = file[:-3].split("-", 1)
        if len(parts) == 2 and parts[0].isdigit():
            adr_num, title_raw = parts
            title = title_raw.replace("-", " ").title()
            display_name = f"ADR {adr_num}: {title}"
        else:
            display_name = file[:-3].replace("-", " ").title()
        nav_content += f"{INDENT_LEVEL_1}* [{display_name}](ADR/{file})\n"

community_pages = [
    ("Code of Conduct", "code_of_conduct.md"),
    ("Contributing", "contributing.md"),
    ("Security", "security.md"),
    ("License", "license.md"),
]
existing = [(t, p) for t, p in community_pages if os.path.exists(os.path.join("docs", p))]
if existing:
    community_index = "community.md"
    landing = ["# Community {#top}\n\n", "Project policies and how to get involved.\n\n"]
    for title, path in existing:
        landing.append(f"- [{title}]({path})\n")
    write_if_changed(Path(community_index), "".join(landing))
    nav_content += "* [Community](community.md)\n"
    for title, path in existing:
        nav_content += f"{INDENT_LEVEL_1}* [{title}]({path})\n"

write_if_changed(NAV_FILE, nav_content)
