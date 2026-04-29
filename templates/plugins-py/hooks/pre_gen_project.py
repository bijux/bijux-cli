from __future__ import annotations

import re
import sys


PLUGIN_NAMESPACE_PATTERN = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
PROJECT_SLUG_PATTERN = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
SEMVER_PATTERN = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?"
    r"(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
)
RESERVED_NAMESPACES = {
    "agent",
    "apps",
    "atlas",
    "cli",
    "completion",
    "dag",
    "dev",
    "dna",
    "doctor",
    "gnss",
    "help",
    "inspect",
    "plugins",
    "rag",
    "rar",
    "repl",
    "vex",
    "version",
}


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def parse_semver(field_name: str, raw: str) -> tuple[int, int, int, tuple[tuple[int, int | str], ...] | None]:
    match = SEMVER_PATTERN.fullmatch(raw)
    if match is None:
        fail(f"{field_name} must be valid semver")

    prerelease = match.group(4)
    prerelease_identifiers: tuple[tuple[int, int | str], ...] | None = None
    if prerelease:
        prerelease_identifiers = tuple(
            (0, int(identifier)) if identifier.isdigit() else (1, identifier)
            for identifier in prerelease.split(".")
        )

    return (
        int(match.group(1)),
        int(match.group(2)),
        int(match.group(3)),
        prerelease_identifiers,
    )


def compare_semver(
    left: tuple[int, int, int, tuple[tuple[int, int | str], ...] | None],
    right: tuple[int, int, int, tuple[tuple[int, int | str], ...] | None],
) -> int:
    left_core = left[:3]
    right_core = right[:3]
    if left_core < right_core:
        return -1
    if left_core > right_core:
        return 1

    left_pre = left[3]
    right_pre = right[3]
    if left_pre is None and right_pre is None:
        return 0
    if left_pre is None:
        return 1
    if right_pre is None:
        return -1

    for left_id, right_id in zip(left_pre, right_pre):
        if left_id == right_id:
            continue
        if left_id[0] != right_id[0]:
            return -1 if left_id[0] < right_id[0] else 1
        return -1 if left_id[1] < right_id[1] else 1

    if len(left_pre) < len(right_pre):
        return -1
    if len(left_pre) > len(right_pre):
        return 1
    return 0


project_slug = "{{ cookiecutter.project_slug }}".strip()
plugin_namespace = "{{ cookiecutter.plugin_namespace }}".strip()
plugin_version = "{{ cookiecutter.plugin_version }}".strip()
cli_min = "{{ cookiecutter.cli_min }}".strip()
cli_max = "{{ cookiecutter.cli_max }}".strip()

if not PROJECT_SLUG_PATTERN.fullmatch(project_slug):
    fail(
        "project_slug must be lowercase kebab-case; pass project_slug explicitly when project_name contains unstable punctuation"
    )

if not PLUGIN_NAMESPACE_PATTERN.fullmatch(plugin_namespace):
    fail(
        "plugin_namespace must be lowercase kebab-case, start with a letter, and avoid repeated hyphens"
    )

if plugin_namespace in RESERVED_NAMESPACES:
    fail("plugin_namespace is reserved by bijux-cli or an official Bijux tool")

cli_min_version = parse_semver("cli_min", cli_min)
cli_max_version = parse_semver("cli_max", cli_max)
parse_semver("plugin_version", plugin_version)
if compare_semver(cli_max_version, cli_min_version) <= 0:
    fail("cli_max must be greater than cli_min")
