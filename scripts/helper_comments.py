# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""A script to find and report all commented lines in a codebase.

This script recursively scans a specified directory for files with given
extensions (e.g., `.py`). It identifies every line containing a hash symbol
(`#`) and treats it as a "violation." The purpose is to gather all comments
from a project to help build an allow-list for linting or quality checks. The
results, including file paths and line numbers, are written to a specified
output file.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import sys


def extract_violations(
    root_input: Path, output_path: Path, extensions: list[str]
) -> None:
    """Scans a directory for files and extracts all lines containing comments.

    This function recursively searches the `root_input` directory for files
    matching the provided `extensions`. For each file found, it reads through
    the content line by line. Any line containing a hash symbol (`#`) is
    recorded along with its line number. The relative path of the file and all
    such lines are written to the `output_path`.

    Args:
        root_input (Path): The root directory to start the scan from.
        output_path (Path): The file path to write the violation report to.
        extensions (list[str]): A list of file extensions to scan (e.g.,
            ['.py', '.pyi']).

    Returns:
        None:
    """
    root_dir = root_input.resolve()
    with output_path.open("w", encoding="utf-8") as out_file:
        for ext in extensions:
            for filepath in root_dir.rglob(f"*{ext}"):
                if not filepath.is_file():
                    continue

                violations: list[tuple[int, str]] = []
                try:
                    for lineno, raw_line in enumerate(
                        filepath.open("r", encoding="utf-8"), start=1
                    ):
                        if "#" not in raw_line:
                            continue
                        line = raw_line.rstrip("\n")
                        violations.append((lineno, line))
                except UnicodeDecodeError:
                    continue

                if violations:
                    rel_path = filepath.relative_to(root_dir)
                    out_file.write(f"{(root_input / rel_path).as_posix()}\n")
                    for lineno, full_line in violations:
                        out_file.write(f"{lineno}: {full_line}\n")
                    out_file.write("\n")


def main() -> None:
    """Parses command-line arguments and orchestrates the comment extraction.

    This is the main entry point for the script when run from the command line.
    It sets up `argparse` to handle inputs for the root directory, output file,
    and file extensions. It performs basic validation and then calls
    `extract_violations` to perform the core logic.
    """
    parser = argparse.ArgumentParser(
        description="Find all Python lines containing '#' not in the allow‑list."
    )
    parser.add_argument("root", help="Root directory to scan (e.g. 'src').")
    parser.add_argument(
        "-o",
        "--output",
        default="comments.txt",
        help="Output text file (default: comments.txt).",
    )
    parser.add_argument(
        "-e",
        "--ext",
        nargs="+",
        default=[".py"],
        help="File extensions to include (default: .py).",
    )

    args = parser.parse_args()
    root_input = Path(args.root)
    if not root_input.is_dir():
        sys.exit(f"Error: '{root_input}' is not a directory.")

    output_path = Path(args.output)
    extract_violations(root_input, output_path, args.ext)
    print(f"Violation lines written to {output_path}")


if __name__ == "__main__":
    main()
