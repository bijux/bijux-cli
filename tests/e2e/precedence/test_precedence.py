# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Precedence E2E tests."""

from __future__ import annotations

import json
import re

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_config_consistent,
    assert_exit_code_stable,
    assert_no_traceback,
)

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _json_value(payload: str) -> str:
    data = json.loads(payload)
    assert isinstance(data, dict)
    return str(data["value"])


def _json_error(payload: str) -> str:
    data = json.loads(payload)
    assert isinstance(data, dict)
    return str(data.get("error", ""))


def _extract_json_payload(text: str) -> str:
    ansi = re.compile(r"\x1b\[[0-9;]*m")
    for line in reversed(text.splitlines()):
        cleaned = ansi.sub("", line).strip()
        if cleaned.startswith("{") and cleaned.endswith("}"):
            return cleaned
    return ""


def _restart(h: E2EHarness) -> E2EHarness:
    return E2EHarness(root=h.root)


@pytest.mark.parametrize(
    ("key", "cfg_val", "env_val"),
    [
        ("prec_a", "from_config", "from_env"),
        ("prec_b", "left", "right"),
    ],
)
def test_env_overrides_config_value(key: str, cfg_val: str, env_val: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}={cfg_val}"])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        env_key = f"BIJUXCLI_{key.upper()}"
        extra_env = {env_key: env_val}
        h2 = _restart(h)
        res_env = h2.run(
            ["config", "get", key, "--format", "json"], extra_env=extra_env
        )
        assert res_env.returncode == 0
        assert_no_traceback(res_env.stdout + res_env.stderr)
        assert _json_value(res_env.stdout) == env_val

        h3 = _restart(h2)
        res_cfg = h3.run(["config", "get", key, "--format", "json"])
        assert res_cfg.returncode == 0
        assert_no_traceback(res_cfg.stdout + res_cfg.stderr)
        assert _json_value(res_cfg.stdout) == cfg_val
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    "missing_key",
    [
        "prec_missing_a",
        "prec_missing_b",
    ],
)
def test_quiet_trace_and_json_preserve_exit_codes(missing_key: str) -> None:
    with E2EHarness() as h:
        res_trace = h.run(["config", "get", missing_key, "--log-level", "trace"])
        assert res_trace.returncode == 2
        assert_no_traceback(res_trace.stdout + res_trace.stderr)
        assert res_trace.stderr.strip() != ""

        h2 = _restart(h)
        res_quiet = h2.run(["config", "get", missing_key, "--quiet"])
        assert res_quiet.returncode == 2
        assert_no_traceback(res_quiet.stdout + res_quiet.stderr)
        assert res_quiet.stdout.strip() == ""

        h3 = _restart(h2)
        res_json = h3.run(
            ["config", "get", missing_key, "--format", "json", "--log-level", "trace"]
        )
        assert res_json.returncode == 2
        assert_no_traceback(res_json.stdout + res_json.stderr)
        payload = _extract_json_payload(res_json.stdout + res_json.stderr)
        assert payload != ""
        assert _json_error(payload) != ""
        assert_exit_code_stable(
            [res_trace.returncode, res_quiet.returncode, res_json.returncode]
        )
        assert_config_consistent(h3)
