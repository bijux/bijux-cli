# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Validate E2E test scope (markers + test count cap)."""

from __future__ import annotations

import ast
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
E2E_DIR = ROOT / "tests" / "e2e"
MIN_TESTS = 100
MAX_TESTS = 150


def _has_marker(node: ast.AST, marker: str) -> bool:
    if isinstance(node, ast.Attribute):
        return node.attr == marker
    if isinstance(node, ast.Call):
        return _has_marker(node.func, marker)
    if isinstance(node, ast.Name):
        return node.id == marker
    if isinstance(node, ast.List | ast.Tuple):
        return any(_has_marker(elt, marker) for elt in node.elts)
    if isinstance(node, ast.Subscript):
        return _has_marker(node.value, marker)
    return False


def _module_markers(tree: ast.Module) -> ast.AST | None:
    for stmt in tree.body:
        if isinstance(stmt, ast.Assign):
            for target in stmt.targets:
                if isinstance(target, ast.Name) and target.id == "pytestmark":
                    return stmt.value
    return None


def _collect_test_funcs(tree: ast.Module) -> list[ast.FunctionDef]:
    return [
        node
        for node in tree.body
        if isinstance(node, ast.FunctionDef) and node.name.startswith("test_")
    ]


def _decorator_has_marker(func: ast.FunctionDef, marker: str) -> bool:
    return any(_has_marker(deco, marker) for deco in func.decorator_list)


def _module_has_marker(module_marker: ast.AST | None, marker: str) -> bool:
    if module_marker is None:
        return False
    return _has_marker(module_marker, marker)


def _parametrize_count(func: ast.FunctionDef) -> int:
    total = 1
    for deco in func.decorator_list:
        if isinstance(deco, ast.Call) and _has_marker(deco.func, "parametrize"):
            if deco.args and isinstance(deco.args[1], ast.List):
                total *= len(deco.args[1].elts)
    return total


def main() -> int:
    errors: list[str] = []
    test_count = 0
    collected = 0
    for path in sorted(E2E_DIR.rglob("test_*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        module_marker = _module_markers(tree)
        test_funcs = _collect_test_funcs(tree)
        test_count += len(test_funcs)
        for func in test_funcs:
            collected += _parametrize_count(func)

        for func in test_funcs:
            has_e2e = _decorator_has_marker(func, "e2e") or _module_has_marker(
                module_marker, "e2e"
            )
            if not has_e2e:
                errors.append(f"{path}: missing @pytest.mark.e2e")

            has_slow = _decorator_has_marker(func, "slow") or _module_has_marker(
                module_marker, "slow"
            )
            if not has_slow:
                errors.append(f"{path}: missing @pytest.mark.slow")

    if collected < MIN_TESTS:
        errors.append(
            f"tests/e2e below minimum: {collected} < {MIN_TESTS}"
        )
    if collected > MAX_TESTS:
        errors.append(
            f"tests/e2e exceeds hard cap: {collected} > {MAX_TESTS}"
        )
    if test_count > MAX_TESTS:
        errors.append(
            f"tests/e2e exceeds hard cap: {test_count} > {MAX_TESTS}"
        )

    if errors:
        print("E2E contract violations:")
        for err in sorted(set(errors)):
            print(f"- {err}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
