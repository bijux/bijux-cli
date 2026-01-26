# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly idempotence stress tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


def _config_size(path: Path) -> int:
    return path.stat().st_size if path.exists() else 0


def test_idempotent_commands_do_not_drift() -> None:
    with E2EHarness() as h:
        key = "idemp_key"
        res = h.run(["config", "set", f"{key}=1"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        baseline = _config_size(h.config_path)

        for _ in range(200):
            res = h.run(["config", "set", f"{key}=1"])
            assert res.returncode == 0
            assert_no_traceback(res.stdout + res.stderr)
            assert_config_consistent(h)

            size = _config_size(h.config_path)
            assert size == baseline
