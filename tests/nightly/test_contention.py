# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly multi-process contention checks."""

from __future__ import annotations

import subprocess  # noqa: S603

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


def test_parallel_reads_do_not_corrupt_state() -> None:
    with E2EHarness() as h:
        cmd = [str(h.bin), "config", "get", "missing_key"]
        p1 = subprocess.Popen(  # noqa: S603 # nosec B603
            cmd, env=h.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )
        p2 = subprocess.Popen(  # noqa: S603 # nosec B603
            cmd, env=h.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )

        out1, err1 = p1.communicate(timeout=20)
        out2, err2 = p2.communicate(timeout=20)

        assert p1.returncode in (1, 2)
        assert p2.returncode in (1, 2)
        assert_no_traceback(out1 + err1)
        assert_no_traceback(out2 + err2)
        assert_config_consistent(h)
