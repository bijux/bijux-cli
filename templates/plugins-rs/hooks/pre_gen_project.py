from __future__ import annotations

import re
import sys


PLUGIN_NAMESPACE_PATTERN = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
CRATE_NAME_PATTERN = re.compile(r"^[a-z][a-z0-9_]*$")
RESERVED_NAMESPACES = {
    "agent",
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


plugin_namespace = "{{ cookiecutter.plugin_namespace }}".strip()
crate_name = "{{ cookiecutter.crate_name }}".strip()

if not PLUGIN_NAMESPACE_PATTERN.fullmatch(plugin_namespace):
    fail(
        "plugin_namespace must be lowercase kebab-case, start with a letter, and avoid repeated hyphens"
    )

if plugin_namespace in RESERVED_NAMESPACES:
    fail("plugin_namespace is reserved by bijux-cli or an official Bijux tool")

if not CRATE_NAME_PATTERN.fullmatch(crate_name):
    fail("crate_name must be lowercase snake_case and start with a letter")
