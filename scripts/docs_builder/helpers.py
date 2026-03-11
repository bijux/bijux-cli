# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Shared helpers for generating files for an MkDocs site.

This module provides a suite of functions designed to assist with the dynamic
generation of Markdown files and navigation for MkDocs, particularly when using
the `mkdocs-gen-files` plugin.

It is designed to be safe to use both within a running `mkdocs build` process
and in standalone scripts (e.g., for local testing or pre-generation).

Key Features:
    - **Safe I/O:** Lazily imports `mkdocs_gen_files` to avoid premature
      configuration probing. It provides a `write_if_changed` function that
      writes to the virtual filesystem when available, but automatically falls
      back to a physical disk location (defaulting to `artifacts/docs/docs`)
      when run outside of a MkDocs build.
    - **Disk Fallback Control:** The disk fallback behavior can be explicitly
      controlled with environment variables:
        - `GEN_FILES_DISK_FALLBACK=1`: Forces all writes to the disk.
        - `GEN_FILES_DISK_DIR`: Overrides the default disk output directory.
    - **Utilities:** Includes pure helper functions for text manipulation,
      link rewriting, and navigation file generation that do not depend on
      the MkDocs environment.

Attributes:
    REPO_ROOT (Path): The absolute path to the repository's root directory.
    SRC_DIR (Path): The path to the main source code directory (`src/bijux_cli`).
    NAV_FILE (Path): The conventional name for the literate navigation file.
    INDENT1 (str): A string representing one level of indentation (4 spaces).
    INDENT2 (str): A string representing two levels of indentation (8 spaces).
    INDENT3 (str): A string representing three levels of indentation (12 spaces).
    INDENT4 (str): A string representing four levels of indentation (16 spaces).
"""

from __future__ import annotations

import json
import os
import re
from pathlib import Path
from typing import Iterable, Match

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
SRC_DIR: Path = Path("src/bijux_cli")
NAV_FILE: Path = Path("nav.md")

INDENT1 = "    "
INDENT2 = INDENT1 * 2
INDENT3 = INDENT1 * 3
INDENT4 = INDENT1 * 4

__all__ = [
    "REPO_ROOT",
    "SRC_DIR",
    "NAV_FILE",
    "INDENT1",
    "INDENT2",
    "INDENT3",
    "INDENT4",
    "fs_read_text",
    "write_if_changed",
    "copy_bytes",
    "copy_tree_into_docs",
    "slug",
    "indent",
    "pretty_title",
    "ensure_top_anchor",
    "anchor_for",
    "rewrite_links_general",
    "rewrite_links_tree",
    "final_fixups",
    "nav_header",
    "nav_add_line",
    "nav_add_bullets",
    "artifacts_group_once",
    "try_json_pretty",
]

_LINK_PAT = re.compile(r"\]\(([^)]+)\)")
_INLINE_LINK = re.compile(r'(!?\[[^\]]*\])\(\s*([^)]+?)\s*(?:(["\'][^)]*["\']))?\s*\)')
_REF_DEF = re.compile(
    r"^(?P<indent>\s*)\[(?P<label>[^\]]+)\]:\s*(?P<url>\S+)(?P<trail>.*)$",
    re.MULTILINE,
)
_AUTOLINK = re.compile(r"<([^ >]+)>")
_DOT_SEG = re.compile(r"^(?:\./|\../)+")
_ANCH_RE = re.compile(r"[^a-z0-9]+")


def _open_mkdocs_fs():
    """Lazily import and return `mkdocs_gen_files.open` if available.

    This function attempts to import `mkdocs_gen_files` only when called. It
    deliberately avoids calling `FilesEditor.current()` or other functions
    that might trigger MkDocs configuration loading, making it safe to use in
    any context. If the import or attribute access fails for any reason, it
    returns `None`.

    Returns:
        The `mkdocs_gen_files.open` function or `None` if not available.
    """
    try:
        import mkdocs_gen_files as gen

        return getattr(gen, "open", None)
    except Exception:
        return None


def _should_use_disk_fallback() -> bool:
    """Check if a disk fallback for file writing is explicitly requested.

    Returns:
        True if the `GEN_FILES_DISK_FALLBACK` environment variable is set to "1".
    """
    return os.environ.get("GEN_FILES_DISK_FALLBACK") == "1"


def _disk_fallback_dir() -> Path:
    """Determine the base directory for disk-based file generation.

    This path is used when `mkdocs_gen_files` is not available or when a disk
    fallback is forced.

    Returns:
        The resolved `Path` object, determined by the `GEN_FILES_DISK_DIR`
        environment variable or defaulting to `artifacts/docs/docs`.
    """
    return Path(os.environ.get("GEN_FILES_DISK_DIR", "artifacts/docs/docs")).resolve()


def fs_read_text(path: Path) -> str:
    """Read a text file from the real filesystem with error tolerance.

    Args:
        path: The path to the file on the physical disk.

    Returns:
        The content of the file as a string, with any decoding errors replaced.
    """
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        return f.read()


def write_if_changed(
    rel_path: Path | str, content: str | bytes, mode: str = "w"
) -> None:
    """Write content to a file, prioritizing the MkDocs virtual filesystem.

    This function provides a robust way to write generated files. It first checks
    if a disk fallback is forced. If not, it attempts to use the virtual
    filesystem provided by `mkdocs_gen_files`. If that fails or is unavailable,
    it safely falls back to writing on the physical disk under a dedicated
    artifacts directory.

    To prevent unnecessary file churn and optimize build times, the function only
    writes the new content if it differs from the existing file's content.

    Args:
        rel_path: The destination path, relative to the docs root.
        content: The string or bytes to write to the file.
        mode: The file open mode (e.g., "w" for text, "wb" for bytes).
    """
    rel_path = Path(rel_path)
    is_bytes = "b" in mode
    read_mode = "rb" if is_bytes else "r"

    if _should_use_disk_fallback():
        out = _disk_fallback_dir() / rel_path
        out.parent.mkdir(parents=True, exist_ok=True)
        existing = (
            out.read_text(encoding="utf-8") if not is_bytes and out.exists() else None
        )
        if existing != content:
            out.write_text(
                content, encoding="utf-8"
            ) if not is_bytes else out.write_bytes(content)
        return

    opener = _open_mkdocs_fs()
    if opener is not None:
        try:
            try:
                with opener(str(rel_path), read_mode) as f:
                    existing = f.read()
            except FileNotFoundError:
                existing = None
            if existing != content:
                with opener(str(rel_path), mode) as f:
                    f.write(content)
            return
        except Exception:
            pass

    out = _disk_fallback_dir() / rel_path
    out.parent.mkdir(parents=True, exist_ok=True)
    existing = (
        out.read_text(encoding="utf-8") if not is_bytes and out.exists() else None
    )
    if existing != content:
        out.write_text(content, encoding="utf-8") if not is_bytes else out.write_bytes(
            content
        )


def copy_bytes(src: Path, dst: str) -> None:
    """Copy a binary file from the real filesystem to the docs destination.

    This is a convenience wrapper around `write_if_changed` for binary files.

    Args:
        src: The source file path on the real filesystem.
        dst: The destination path, relative to the docs root.
    """
    data = Path(src).read_bytes()
    write_if_changed(Path(dst), data, mode="wb")


def copy_tree_into_docs(src: Path, dst_root: str) -> None:
    """Recursively copy a directory from the real filesystem into the docs.

    Args:
        src: The source directory path on the real filesystem.
        dst_root: The destination directory, relative to the docs root.
    """
    src = Path(src)
    if not src.is_dir():
        return
    for p in src.rglob("*"):
        if p.is_file():
            rel = p.relative_to(src).as_posix()
            copy_bytes(p, f"{dst_root}/{rel}")


def slug(stem: str) -> str:
    """Convert a string into a URL-friendly slug.

    The conversion involves making the string lowercase, replacing sequences of
    non-alphanumeric characters with a single hyphen, and removing any leading
    or trailing hyphens.

    Args:
        stem: The input string.

    Returns:
        The slugified string.
    """
    return re.sub(r"[^a-z0-9]+", "-", stem.lower()).strip("-")


def indent(text: str, n: int = 4) -> str:
    """Indent each line of a multi-line string.

    Args:
        text: The string to indent.
        n: The number of spaces to use for indentation. Defaults to 4.

    Returns:
        The indented string.
    """
    pad = " " * n
    return "".join(
        pad + line if line.strip() else pad + "\n" for line in text.splitlines(True)
    )


def pretty_title(stem: str) -> str:
    """Convert a snake_case or kebab-case string to a title.

    Args:
        stem: The input string (e.g., "my_file_name").

    Returns:
        A title-cased string (e.g., "My File Name").
    """
    return stem.replace("_", " ").title()


def ensure_top_anchor(md: str) -> str:
    """Ensure a Markdown string has a `{#top}` anchor on its first heading.

    This function scans the beginning of a Markdown document for a level 1
    heading (`# `). If found, it appends a `{#top}` anchor to it. If no such
    heading is found near the top, it prepends an HTML `<a id="top"></a>` tag.
    It does nothing if an anchor is already present.

    Args:
        md: The input Markdown content.

    Returns:
        The Markdown content with a top anchor guaranteed.
    """
    if "{#top}" in md or 'id="top"' in md or "(#top)" in md:
        return md
    lines = md.splitlines()
    for i, line in enumerate(lines[:20]):
        if line.startswith("# "):
            lines[i] = line.rstrip() + " {#top}"
            return "\n".join(lines)
    return '<a id="top"></a>\n\n' + md


def anchor_for(label: str) -> str:
    """Generate a Markdown-compatible anchor slug from a string.

    Args:
        label: The string to convert into an anchor.

    Returns:
        The generated anchor string.
    """
    s = Path(label).as_posix().lower()
    return _ANCH_RE.sub("-", s).strip("-")


def _normalize_rel(href: str) -> str:
    """Remove leading relative path segments like `./` or `../`.

    Args:
        href: The input path string.

    Returns:
        The path string with leading relative segments removed.
    """
    return _DOT_SEG.sub("", href)


def _rewrite_url(url: str, target_map: dict[str, str]) -> str:
    """Rewrite a URL based on a mapping, preserving fragments and absolute URLs.

    Args:
        url: The URL to potentially rewrite.
        target_map: A dictionary mapping source paths to target paths.

    Returns:
        The rewritten URL, or the original URL if no changes were needed.
    """
    base, sep, frag = url.partition("#")
    if re.match(r"^[a-zA-Z][a-zA-Z0-9+\-.]*:", base):
        return url
    base_norm = _normalize_rel(base)
    mapped = target_map.get(base_norm)
    resolved = mapped if mapped is not None else base
    return resolved + (sep + frag if frag else "")


def rewrite_links_general(md: str) -> str:
    """Perform general-purpose link rewriting in a Markdown string.

    This function rewrites links based on a predefined mapping for common
    project files (e.g., `docs/index.md`). It handles various Markdown link
    formats, including inline links, reference-style links, and autolinks.

    Args:
        md: The input Markdown content.

    Returns:
        The Markdown content with links rewritten.
    """
    target_map = {
        "docs/index.md": "index.md",
    }
    md = _INLINE_LINK.sub(
        lambda m: f"{m.group(1)}({_rewrite_url(m.group(2).strip(), target_map)}"
        f"{(' ' + m.group(3)) if m.group(3) else ''})",
        md,
    )
    md = _REF_DEF.sub(
        lambda m: f"{m.group('indent')}[{m.group('label')}]: "
        f"{_rewrite_url(m.group('url'), target_map)}{m.group('trail')}",
        md,
    )
    md = _AUTOLINK.sub(lambda m: f"<{_rewrite_url(m.group(1), target_map)}>", md)
    return md


def rewrite_links_tree(md: str) -> str:
    """Rewrite links specifically for the project tree and source code files.

    This function extends `rewrite_links_general` with rules tailored to this
    project. It converts links pointing to source paths into GitHub URLs so
    the docs site can link to the code directly.

    Args:
        md: The input Markdown content.

    Returns:
        The Markdown content with source links rewritten for the docs site.
    """
    md = rewrite_links_general(md)

    repo_base = "https://github.com/bijux/bijux-cli/blob/main/"

    def repl(m: Match[str]) -> str:
        href = m.group(1)
        if href.startswith("src/"):
            return f"]({repo_base}{href})"
        if href in (
            "#source-code-srcbijux_cli",
            "#templates-templates",
        ):
            return "](#top)"
        return m.group(0)

    md = _LINK_PAT.sub(repl, md)
    return md


def final_fixups(md: str) -> str:
    """Perform a final pass of simple string replacements to fix common links.

    Args:
        md: The input Markdown content.

    Returns:
        The cleaned-up Markdown content.
    """
    return md


def nav_header() -> str:
    """Return the standard header for the literate navigation file.

    Returns:
        The header string "# Full Navigation\n".
    """
    return "# Full Navigation\n"


def nav_add_line(nav: str, line: str) -> str:
    """Append a line to a navigation string, ensuring proper newlines.

    Args:
        nav: The existing navigation content.
        line: The line to append.

    Returns:
        The updated navigation content.
    """
    if not nav.endswith("\n"):
        nav += "\n"
    return nav + line + ("\n" if not line.endswith("\n") else "")


def nav_add_bullets(nav: str, lines: Iterable[str]) -> str:
    """Append an iterable of lines as bullet points to the navigation string.

    Args:
        nav: The existing navigation content.
        lines: An iterable of strings to append as new lines.

    Returns:
        The updated navigation content.
    """
    for ln in lines:
        nav = nav_add_line(nav, ln)
    return nav


def artifacts_group_once(nav: str) -> str:
    """Add the '* Artifacts' group header to the navigation if not present.

    This function ensures that the "Artifacts" group is only added once,
    making it safe to call multiple times.

    Args:
        nav: The existing navigation content.

    Returns:
        The updated navigation content, with the "Artifacts" group guaranteed.
    """
    return nav if "* Artifacts\n" in nav else nav + "* Artifacts\n"


def try_json_pretty(text: str) -> str:
    """Attempt to format a string as pretty-printed JSON.

        If the string is valid JSON, it is parsed and returned as an indented
        string. If it is not valid JSON, the original string is returned, making
    ax the function safe to use on arbitrary text.

        Args:
            text: The input string, which may or may not be JSON.

        Returns:
            A pretty-printed JSON string or the original string.
    """
    try:
        obj = json.loads(text) if text else None
    except json.JSONDecodeError:
        return text
    return json.dumps(obj, indent=2, ensure_ascii=False) if obj is not None else text
