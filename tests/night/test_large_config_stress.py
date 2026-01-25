# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly large-scale config stress test."""

from __future__ import annotations

import time

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback

pytestmark = [pytest.mark.e2e, pytest.mark.night]


def test_large_config_list_is_stable() -> None:
    with E2EHarness() as h:
        lines = [f"BIJUXCLI_KEY{i}=value{i}" for i in range(1000)]
        h.config_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

        start = time.perf_counter()
        res = h.run(["config", "list", "--format", "json"])
        elapsed = time.perf_counter() - start

        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        lines = [
            line
            for line in h.config_path.read_text(encoding="utf-8").splitlines()
            if line
        ]
        assert len(lines) >= 1000
        assert elapsed < 3.0
        assert_config_consistent(h)
