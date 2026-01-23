# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Architecture tests for infra contracts."""

from __future__ import annotations

from pathlib import Path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def test_no_infra_contracts_in_core() -> None:
    """Core must not define infra contracts."""
    core_contracts = (
        Path(__file__).resolve().parents[3] / "src/bijux_cli/core/contracts.py"
    )
    text = _read_text(core_contracts)
    assert "Serializer" not in text
    assert "RetryPolicy" not in text
    assert "Emitter" not in text
    assert "ProcessRunner" not in text


def test_no_imports_from_core_contracts_for_infra() -> None:
    """Infra contracts must be sourced from bijux_cli.infra.contracts."""
    root = Path(__file__).resolve().parents[3] / "src"
    for path in root.rglob("*.py"):
        text = _read_text(path)
        assert "from bijux_cli.core.contracts import Emitter" not in text
        assert "from bijux_cli.core.contracts import Serializer" not in text
        assert "from bijux_cli.core.contracts import RetryPolicy" not in text
        assert "from bijux_cli.core.contracts import ProcessRunner" not in text
