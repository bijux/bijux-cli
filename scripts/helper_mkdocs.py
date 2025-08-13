# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Dynamically generates MkDocs API reference pages and navigation.

This script is designed to be executed by the `mkdocs-gen-files` plugin
during the MkDocs build process. It automates the creation of the
documentation site's structure by performing two main tasks:

1.  **API Page Generation**: It walks through the `src/bijux_cli` source
    directory and creates a corresponding Markdown file for each Python module.
    Each of these files is populated with a `mkdocstrings` handler that
    renders the module's docstrings into a complete API reference page.

2.  **Navigation File Generation**: It constructs a complete navigation map for
    the site in a format compatible with the `mkdocs-literate-nav` plugin.
    This map includes static pages (Home, User Guide), the generated API
    reference pages (nested by source directory), a link to the changelog,
    and a sorted list of Architecture Decision Records (ADRs).

This process ensures that the site's API documentation and navigation always
stay in sync with the project's source code structure.
"""

from __future__ import annotations

import os

import mkdocs_gen_files

SRC_DIR = "src/bijux_cli"
ADR_DIR = "docs/ADR"
NAV_FILE = "nav.md"

INDENT_LEVEL_1 = "    "
INDENT_LEVEL_2 = INDENT_LEVEL_1 * 2

nav_content = "# Full Navigation\n"
nav_content += "* [Home](index.md)\n"
nav_content += "* [User Guide](usage.md)\n"
nav_content += "* API Reference\n"

api_sections: dict[str, list[tuple[str, str]]] = {}

for root, _, files in os.walk(SRC_DIR):
    rel_root = os.path.relpath(root, SRC_DIR)
    section = rel_root if rel_root != "." else None

    for file in files:
        if file.endswith(".py") and not file.startswith("__") and file != "py.typed":
            module_name = file[:-3]
            md_path = f"reference/{rel_root}/{module_name}.md".replace(
                "\\", "/"
            ).lstrip("./")

            with mkdocs_gen_files.open(md_path, "w") as f:
                if "commands" in rel_root:
                    f.write(f"# {module_name.capitalize()} Command API Reference\n")
                    f.write(
                        "This section documents the internals of the "
                        f"`{module_name}` command in Bijux CLI, including all "
                        "arguments, options, and output structure.\n"
                    )
                else:
                    f.write(f"# {module_name.capitalize()} Module API Reference\n")
                    f.write(
                        "This section documents the internals of the "
                        f"`{module_name}` module in Bijux CLI.\n"
                    )

                if section is None:
                    full_module_path = f"bijux_cli.{module_name}"
                else:
                    full_module_path = (
                        f"bijux_cli.{rel_root.replace(os.sep, '.')}.{module_name}"
                    )

                f.write(f"::: {full_module_path}\n")
                f.write("    handler: python\n")
                f.write("    options:\n")
                f.write("      show_root_heading: true\n")
                f.write("      show_source: true\n")
                f.write("      show_signature_annotations: true\n")
                f.write("      docstring_style: google\n")

            display_name = (
                f"{module_name.capitalize()} Command"
                if "commands" in rel_root
                else f"{module_name.capitalize()} Module"
            )
            md_link = md_path

            if section:
                if section not in api_sections:
                    api_sections[section] = []
                api_sections[section].append((display_name, md_link))
            else:
                nav_content += f"{INDENT_LEVEL_1}* [{display_name}]({md_link})\n"

for subsection, items in sorted(api_sections.items()):
    if not items:
        continue
    nav_content += f"{INDENT_LEVEL_1}* {subsection.replace(os.sep, '/').capitalize()}\n"
    for display_name, md_link in sorted(items):
        nav_content += f"{INDENT_LEVEL_2}* [{display_name}]({md_link})\n"

nav_content += "* [Changelog](changelog.md)\n"
nav_content += "* Architecture Decision Records\n"
nav_content += f"{INDENT_LEVEL_1}* [Overview](ADR/index.md)\n"

adr_files = []
if os.path.isdir(ADR_DIR):
    adr_files = [
        file
        for file in os.listdir(ADR_DIR)
        if file.endswith(".md") and file != "index.md"
    ]

adr_files.sort()

for file in adr_files:
    parts = file[:-3].split("-", 1)
    if len(parts) == 2 and parts[0].isdigit():
        adr_num = parts[0]
        title = parts[1].replace("-", " ").title()
        display_name = f"ADR {adr_num}: {title}"
    else:
        display_name = file[:-3].replace("-", " ").title()

    md_link = f"ADR/{file}"
    nav_content += f"{INDENT_LEVEL_1}* [{display_name}]({md_link})\n"

with mkdocs_gen_files.open(NAV_FILE, "w") as f:
    f.write(nav_content)
