# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Nightly plugin + precedence interaction stress."""

from __future__ import annotations

from hypothesis import given, settings
from hypothesis import strategies as st
import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import assert_no_traceback, assert_plugins_consistent
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.nightly]


@settings(max_examples=20, deadline=None)
@given(
    use_quiet=st.booleans(),
    use_json=st.booleans(),
    log_level=st.sampled_from(["trace", "debug", "info"]),
)
def test_plugin_commands_respect_precedence(
    use_quiet: bool, use_json: bool, log_level: str
) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / "prec_plugin", name="prec_plugin")
        res_install = h.run(["plugins", "install", str(dummy_dir)])
        assert res_install.returncode == 0
        assert_no_traceback(res_install.stdout + res_install.stderr)

        args = ["plugins", "list"]
        if use_json:
            args += ["--format", "json"]
        if use_quiet:
            args.append("--quiet")
        args += ["--log-level", log_level]

        res = h.run(args)
        assert res.returncode in (0, 1, 2)
        assert_no_traceback(res.stdout + res.stderr)
        if use_quiet:
            assert res.stdout.strip() == ""
        assert_plugins_consistent(h)
