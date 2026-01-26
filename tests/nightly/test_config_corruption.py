# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly config corruption resistance."""

from __future__ import annotations

import random

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_config_consistent, assert_no_traceback

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


def _mutate_config_lines(lines: list[str]) -> list[str]:
    if not lines:
        return ["BIJUXCLI_SEED=1"]
    lines = lines[:]
    random.shuffle(lines)
    lines.append("BAD_LINE_NO_EQUALS")
    lines.append("BIJUXCLI_DUP=1")
    lines.append("BIJUXCLI_DUP=2")
    return lines


@settings(max_examples=10, deadline=None)
@given(
    extra=st.lists(
        st.text(
            alphabet=st.characters(
                min_codepoint=32,
                max_codepoint=126,
                blacklist_characters=["\n", "\r"],
            ),
            min_size=1,
            max_size=10,
        ),
        min_size=1,
        max_size=5,
    )
)
def test_config_corruption_resistance(extra: list[str]) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", "seed=1"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        lines = h.config_path.read_text(encoding="utf-8").splitlines()
        mutated = _mutate_config_lines(lines + extra)
        h.config_path.write_text("\n".join(mutated) + "\n", encoding="utf-8")
        before = h.config_path.read_text(encoding="utf-8")

        res_get = h.run(["config", "get", "seed"])
        assert res_get.returncode in (0, 1, 2)
        assert_no_traceback(res_get.stdout + res_get.stderr)
        assert h.config_path.read_text(encoding="utf-8") == before

        res_set = h.run(["config", "set", "recovered=1"])
        assert res_set.returncode == 0
        assert_no_traceback(res_set.stdout + res_set.stderr)
        assert_config_consistent(h)
