# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a framework for generating Markdown documentation from artifact files.

This module contains a base class, `StandardArtifactPage`, designed to create
standardized documentation pages. It includes helpers for formatting content,
such as creating URL-friendly anchors and structured bullet points.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from scripts.docs_builder.helpers import anchor_for
from scripts.docs_builder.helpers import indent
from scripts.docs_builder.helpers import try_json_pretty
from scripts.docs_builder.helpers import write_if_changed


@dataclass(frozen=True)
class Bullet:
    """A data structure representing a single bullet point in a summary list.

    Attributes:
        title: The main line of the bullet point, typically short and containing
            a link, e.g., "[filename](#anchor) — Description".
        good: An optional line describing a "good" state or a success metric,
            e.g., "0 violations (Google convention) — violations: 0".
        usage: An optional line providing usage instructions or a call to action,
            e.g., "Run `ruff format`; enforce via pre-commit."
    """
    title: str
    good: str | None = None
    usage: str | None = None


def bullet_block(title: str, good: str | None = None, usage: str | None = None) -> str:
    """Formats a bullet's components into a Markdown list item string.

    Args:
        title: The main title for the bullet point.
        good: Optional text for the 'Good' sub-bullet.
        usage: Optional text for the 'How to use' sub-bullet.

    Returns:
        A multi-line string representing the formatted Markdown bullet block.
    """
    out = f"- {title}\n"
    if good:
        out += f"    - **Good:** {good}\n"
    if usage:
        out += f"    - **How to use:** {usage}\n"
    return out + "\n"


class StandardArtifactPage:
    """A base class for generating a standard artifact documentation page.

    This class provides a skeleton for a Markdown page that includes a title,
    an introduction, a summary list of artifacts, and detailed sections for each
    artifact. Subclasses must provide the content by implementing the abstract
    methods.

    The generated page structure is as follows:
    - H1 Title with a 'top' anchor.
    - An introductory 'tip' block.
    - A bulleted list summarizing each file and its purpose.
    - A horizontal rule separator.
    - A detailed section for each file, including an optional description and
        a link to view the artifact's raw contents (opened in a new browser tab
        to keep the main page focused).

    Attributes:
        out_md: The path to the output Markdown file that will be generated.
    """

    out_md: Path

    def title(self) -> str:
        """Provides the main H1 title for the documentation page.

        Subclasses must implement this method.

        Returns:
            The string to be used as the page title.
        """
        ...

    def intro(self) -> str:
        """Provides the introductory text for the page.

        This text will be placed inside a 'tip' admonition block.
        Subclasses must implement this method.

        Returns:
            A string containing the introductory content.
        """
        ...

    def iter_items(self) -> Iterable[tuple[str, Path]]:
        """Yields the items to be documented on the page.

        Subclasses must implement this method to iterate through the source
        artifacts.

        Yields:
            Tuples of (label, path), where 'label' is the display name for the
            artifact and 'path' is the `pathlib.Path` to its file or directory.
        """
        ...

    def bullet_for(self, label: str, path: Path, content: str) -> Bullet:
        """Creates a summary bullet point for a given artifact.

        Subclasses must implement this method to define the summary that appears
        in the list at the top of the page.

        Args:
            label: The display name of the artifact.
            path: The path to the artifact's file.
            content: The string content of the artifact file. For directories,
                this will be an empty string.

        Returns:
            A `Bullet` object containing the summary information.
        """
        ...

    def detail_for(self, label: str, path: Path, content: str) -> str:
        """Creates the detailed description for a given artifact.

        Subclasses must implement this method to provide the detailed "About"
        section for each artifact on the page.

        Args:
            label: The display name of the artifact.
            path: The path to the artifact's file.
            content: The string content of the artifact file. For directories,
                this will be an empty string.

        Returns:
            A string (can be multi-line Markdown) for the artifact's
            detailed description.
        """
        ...

    def _raw_root(self) -> Path:
        """Determines the root directory for storing raw artifact content files.

        This path is relative to the `docs/` root and is used by `mkdocs-gen-files`
        as the destination for writing sidecar content files.

        Returns:
            The path for the raw content directory, e.g., `artifacts/_files/lint`.
        """
        section = self.out_md.stem
        return Path("artifacts") / "_files" / section

    def build(self) -> None:
        """Generates the main artifact page and sidecar files for raw content.

        This method orchestrates the page generation. It reads each artifact
        only once. For each item yielded by `iter_items`:
        1.  A summary bullet point is generated for the top of the page.
        2.  A detailed "About" section is created.
        3.  If the item is a file, its raw or pretty-printed content is written
            to a separate "sidecar" file inside the virtual docs' filesystem. A
            link to this file is added to the page, avoiding embedding large
            files directly in the HTML.
        4.  If the item is a directory, its handling is deferred to the specific
            subclass's `detail_for` method.

        Finally, it assembles these parts into the final Markdown page and writes
        it to the path specified by `self.out_md`.
        """
        raw_root = self._raw_root()
        page: list[str] = [f"# {self.title()} {{#top}}\n\n"]
        page.append('!!! tip "What are these files?"\n\n')
        page.append(indent(self.intro()))
        page.append("\n")

        items = list(self.iter_items())
        if not items:
            page.append("_No files found._\n")
            write_if_changed(self.out_md, "".join(page))
            return

        bullets_content: list[str] = []
        details_content: list[str] = []
        section = self.out_md.stem
        link_root_rel = Path(".") / "_files" / section

        for label, path in items:
            exists = path.exists()
            content = self._read(path) if exists else ""

            bullet = self.bullet_for(label, path, content)
            bullet_text = bullet_block(bullet.title, bullet.good, bullet.usage)
            if not exists:
                bullet_text = bullet_text.rstrip() + "\n    - **Status:** _missing at build time_\n\n"
            bullets_content.append(bullet_text)

            section_md: list[str] = []
            section_md.append(f"## {label} {{#{anchor_for(label)}}}\n\n")
            about = self.detail_for(label, path, content)
            if about:
                section_md.append(about)
                section_md.append("\n")

            if not exists:
                section_md.append('!!! warning "Not found"\n\n')
                section_md.append(
                    indent("This artifact was not present during the build. "
                           "Generate it via the corresponding `make` target (e.g. `make test`, `make lint`).\n"))
                section_md.append("\n")

            if exists and path.is_file():
                rel_path_str = Path(label).as_posix()
                safe_rel = Path(
                    "/".join(part or "file" for part in rel_path_str.split("/")))
                suffix = path.suffix.lower()

                canonical_rel = safe_rel if suffix else safe_rel.with_suffix(".txt")

                body = try_json_pretty(content) if suffix == ".json" else content

                write_target = raw_root / canonical_rel
                write_if_changed(write_target.as_posix(), body, mode="w")

                link_target_rel = link_root_rel / canonical_rel
                if suffix == ".xml":
                    xml_txt_rel = safe_rel.with_suffix(
                        safe_rel.suffix + ".txt")
                    write_if_changed((raw_root / xml_txt_rel).as_posix(), body,
                                     mode="w")
                    link_target_rel = link_root_rel / xml_txt_rel

                section_md.append(
                    f'[Open full contents]({link_target_rel.as_posix()}){{: target="_blank" rel="noopener" }}\n\n'
                )

            details_content.append("".join(section_md))

        page.append("**Files & what they’re for**\n\n")
        page.extend(bullets_content)
        page.append("\n---\n\n")
        page.extend(details_content)

        write_if_changed(self.out_md, "".join(page))

    @staticmethod
    def _read(p: Path) -> str:
        """Safely reads the content of a text file or returns "" for a directory.

        Args:
            p: The `pathlib.Path` object pointing to the file or directory.

        Returns:
            The content of the file as a UTF-8 decoded string. Returns an
            empty string if the path is a directory or an error occurs.
        """
        if not p.is_file():
            return ""
        try:
            return p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return ""
