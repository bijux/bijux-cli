# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly plugin lifecycle abuse tests."""

from __future__ import annotations

import random

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_no_traceback,
    assert_plugins_consistent,
)
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


@settings(max_examples=10, deadline=None)
@given(
    st.lists(
        st.sampled_from(["install", "uninstall", "install_bad"]),
        min_size=20,
        max_size=60,
    )
)
def test_plugin_lifecycle_abuse(ops: list[str]) -> None:
    with E2EHarness() as h:
        names = ["abuse_a", "abuse_b", "abuse_c"]
        for name in names:
            write_dummy_plugin(h.root / name, name=name)

        rng = random.Random(0)  # noqa: S311
        for op in ops:
            name = rng.choice(names)  # noqa: S311
            if op == "install":
                res = h.run(["plugins", "install", str(h.root / name)])
            elif op == "uninstall":
                res = h.run(["plugins", "uninstall", name])
            else:
                bad_dir = h.root / f"{name}_bad"
                bad_dir.mkdir(exist_ok=True)
                (bad_dir / "plugin.py").write_text(
                    "def setup():\n    return None\n", encoding="utf-8"
                )
                (bad_dir / "plugin.json").write_text("{bad json", encoding="utf-8")
                res = h.run(["plugins", "install", str(bad_dir)])

            assert res.returncode in (0, 1, 2)
            assert_no_traceback(res.stdout + res.stderr)
            assert_plugins_consistent(h)
