# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly REPL stress test."""

from __future__ import annotations

import random

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback
from tests.regression.test_functional import run_repl_script

pytestmark = [pytest.mark.e2e, pytest.mark.night]


def test_repl_long_session() -> None:
    with E2EHarness() as h:
        commands = []
        keys = ["r1", "r2", "r3", "r4"]
        rng = random.Random(0)  # noqa: S311
        for _ in range(220):
            choice = rng.choice(["set", "unset", "get", "list", "invalid"])  # noqa: S311
            key = rng.choice(keys)  # noqa: S311
            if choice == "set":
                commands.append(f"config set {key}=1")
            elif choice == "unset":
                commands.append(f"config unset {key}")
            elif choice == "get":
                commands.append(f"config get {key}")
            elif choice == "list":
                commands.append("config list")
            else:
                commands.append("config set")
        commands.append("quit")

        res = run_repl_script(commands, env=h.env, cwd=h.root, timeout=10)
        assert res.returncode in (0, 1, 2)
        assert_no_traceback(res.stdout + res.stderr)
        assert_config_consistent(h)
