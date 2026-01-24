# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Cheap import-boundary tests to prevent architectural drift."""

from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "src" / "bijux_cli"


def _imports_in(path: Path) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"))
    modules: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for name in node.names:
                modules.add(name.name)
        elif isinstance(node, ast.ImportFrom) and node.module:
            modules.add(node.module)
    return modules


def _collect_modules(root: Path) -> dict[Path, set[str]]:
    modules: dict[Path, set[str]] = {}
    for path in root.rglob("*.py"):
        modules[path] = _imports_in(path)
    return modules


def _assert_no_prefix_imports(
    modules: dict[Path, set[str]], prefixes: tuple[str, ...]
) -> None:
    violations: list[str] = []
    for path, imports in modules.items():
        violations.extend(
            f"{path}: {mod}" for mod in imports if mod.startswith(prefixes)
        )
    assert not violations, "Forbidden imports found:\n" + "\n".join(violations)


def test_core_has_no_infra_imports() -> None:
    core_modules = _collect_modules(SRC / "core")
    _assert_no_prefix_imports(core_modules, ("bijux_cli.infra",))


def test_infra_has_no_core_or_services_imports() -> None:
    infra_modules = _collect_modules(SRC / "infra")
    filtered: dict[Path, set[str]] = {}
    for path, imports in infra_modules.items():
        filtered[path] = {mod for mod in imports if mod != "bijux_cli.core.enums"}
    _assert_no_prefix_imports(filtered, ("bijux_cli.core", "bijux_cli.services"))


def test_cli_has_no_infra_imports() -> None:
    cli_modules = _collect_modules(SRC / "cli")
    filtered: dict[Path, set[str]] = {}
    for path, imports in cli_modules.items():
        filtered[path] = {mod for mod in imports if mod != "bijux_cli.infra.contracts"}
    _assert_no_prefix_imports(filtered, ("bijux_cli.infra",))


def test_cli_commands_have_no_raw_flag_strings() -> None:
    forbidden = (
        "--quiet",
        "--log-level",
        "--format",
        "--log-level debug",
        "--pretty",
        "--no-pretty",
        "--color",
        "NO_COLOR",
    )
    roots = [
        SRC / "cli" / "commands",
        SRC / "cli" / "plugins" / "commands",
    ]
    violations: list[str] = []
    for root in roots:
        for path in root.rglob("*.py"):
            content = path.read_text(encoding="utf-8")
            violations.extend(
                f"{path}: {token}" for token in forbidden if token in content
            )
    assert not violations, "Raw flag strings found:\n" + "\n".join(violations)


def test_policy_resolution_lives_in_core_precedence() -> None:
    """Ensure CLI/infra/services do not resolve policy directly."""
    forbidden = (
        "resolve_effective_config",
        "resolve_log_policy",
        "resolve_output_flags",
    )
    roots = [
        SRC / "cli" / "commands",
        SRC / "cli" / "plugins" / "commands",
        SRC / "infra",
        SRC / "services",
    ]
    violations: list[str] = []
    for root in roots:
        for path in root.rglob("*.py"):
            content = path.read_text(encoding="utf-8")
            violations.extend(
                f"{path}: {token}" for token in forbidden if token in content
            )
    assert not violations, "Policy resolution outside core:\n" + "\n".join(violations)
