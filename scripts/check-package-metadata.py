#!/usr/bin/env python3
"""Validate metadata consistency across Cargo and Python package definitions."""

from __future__ import annotations

import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - local runtime fallback
    import tomli as tomllib  # type: ignore[no-redef]


def load_toml(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    workspace = load_toml(root / "Cargo.toml")
    pyproject = load_toml(root / "pyproject.toml")

    expected_repository = workspace["workspace"]["package"]["repository"]
    expected_author = workspace["workspace"]["package"]["authors"][0]
    expected_license_id = workspace["workspace"]["package"]["license"]

    project = pyproject["project"]
    py_repository = project["urls"]["Homepage"]
    py_author = project["authors"][0]["name"]
    py_license_file = project["license"]["file"]
    py_name = project["name"]

    failures: list[str] = []
    if py_name != "bijux-cli":
        failures.append(f"project.name expected 'bijux-cli', got '{py_name}'")
    if py_repository.removesuffix(".git") != expected_repository:
        failures.append(
            f"repository mismatch: pyproject '{py_repository}' vs cargo '{expected_repository}'"
        )
    if py_author != expected_author:
        failures.append(f"author mismatch: pyproject '{py_author}' vs cargo '{expected_author}'")
    if expected_license_id not in py_license_file:
        failures.append(
            f"license mismatch: expected '{expected_license_id}' identifier in '{py_license_file}'"
        )

    if failures:
        for failure in failures:
            print(f"ERROR: {failure}", file=sys.stderr)
        return 1

    print("metadata consistency check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
