# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Precedence E2E tests."""

from __future__ import annotations

import json
import os
import re

import pytest

from tests.e2e.harness import E2EHarness

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


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


@pytest.mark.parametrize(
    ("key", "cfg_val", "env_val"),
    [
        ("prec_a", "from_config", "from_env"),
        ("prec_b", "left", "right"),
        ("prec_c", "low", "high"),
    ],
)
def test_env_overrides_config_value(key: str, cfg_val: str, env_val: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}={cfg_val}"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)

        env_key = f"BIJUXCLI_{key.upper()}"
        extra_env = {env_key: env_val}
        res_env = h.run(["config", "get", key, "--format", "json"], extra_env=extra_env)
        assert res_env.returncode == 0
        _no_traceback(res_env.stdout + res.stderr)
        assert _json_value(res_env.stdout) == env_val

        res_cfg = h.run(["config", "get", key, "--format", "json"])
        assert res_cfg.returncode == 0
        _no_traceback(res_cfg.stdout + res_cfg.stderr)
        assert _json_value(res_cfg.stdout) == cfg_val


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("prec_d", "one"),
        ("prec_e", "two"),
    ],
)
def test_explicit_config_persists_without_env(key: str, value: str) -> None:
    with E2EHarness() as h:
        res = h.run(["config", "set", f"{key}={value}"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)

        env_key = f"BIJUXCLI_{key.upper()}"
        assert env_key not in os.environ

        res_cfg = h.run(["config", "get", key, "--format", "json"])
        assert res_cfg.returncode == 0
        _no_traceback(res_cfg.stdout + res_cfg.stderr)
        assert _json_value(res_cfg.stdout) == value


@pytest.mark.parametrize(
    "missing_key",
    [
        "prec_missing_a",
        "prec_missing_b",
        "prec_missing_c",
    ],
)
def test_format_json_preserves_exit_code_for_missing_key(missing_key: str) -> None:
    with E2EHarness() as h:
        res_plain = h.run(["config", "get", missing_key])
        assert res_plain.returncode == 2
        _no_traceback(res_plain.stdout + res_plain.stderr)
        assert (res_plain.stdout + res_plain.stderr).strip() != ""

        res_json = h.run(["config", "get", missing_key, "--format", "json"])
        assert res_json.returncode == 2
        _no_traceback(res_json.stdout + res_json.stderr)
        payload = _extract_json_payload(res_json.stdout + res_json.stderr)
        assert payload != ""
        assert _json_error(payload) != ""


@pytest.mark.parametrize(
    "missing_key",
    [
        "prec_quiet_a",
        "prec_quiet_b",
    ],
)
def test_quiet_suppresses_output_not_exit_code(missing_key: str) -> None:
    with E2EHarness() as h:
        res_loud = h.run(["config", "get", missing_key])
        assert res_loud.returncode == 2
        _no_traceback(res_loud.stdout + res_loud.stderr)
        assert (res_loud.stdout + res_loud.stderr).strip() != ""

        res_quiet = h.run(["config", "get", missing_key, "--quiet"])
        assert res_quiet.returncode == 2
        _no_traceback(res_quiet.stdout + res_quiet.stderr)
        assert res_quiet.stdout.strip() == ""


@pytest.mark.parametrize(
    "missing_key",
    [
        "prec_trace_a",
        "prec_trace_b",
    ],
)
def test_log_level_trace_does_not_change_exit_code(missing_key: str) -> None:
    with E2EHarness() as h:
        res_trace = h.run(["config", "get", missing_key, "--log-level", "trace"])
        assert res_trace.returncode == 2
        _no_traceback(res_trace.stdout + res_trace.stderr)
        assert (res_trace.stdout + res_trace.stderr).strip() != ""


@pytest.mark.parametrize(
    "missing_key",
    [
        "prec_quiet_trace_a",
        "prec_quiet_trace_b",
    ],
)
def test_quiet_overrides_trace_output(missing_key: str) -> None:
    with E2EHarness() as h:
        res_quiet = h.run(
            ["config", "get", missing_key, "--log-level", "trace", "--quiet"]
        )
        assert res_quiet.returncode == 2
        _no_traceback(res_quiet.stdout + res_quiet.stderr)
        assert res_quiet.stdout.strip() == ""


@pytest.mark.parametrize(
    "missing_key",
    [
        "prec_json_trace_a",
        "prec_json_trace_b",
    ],
)
def test_format_json_with_trace_preserves_exit_code(missing_key: str) -> None:
    with E2EHarness() as h:
        res_json = h.run(
            ["config", "get", missing_key, "--format", "json", "--log-level", "trace"]
        )
        assert res_json.returncode == 2
        _no_traceback(res_json.stdout + res_json.stderr)
        payload = _extract_json_payload(res_json.stdout + res_json.stderr)
        assert payload != ""
        assert _json_error(payload) != ""
