# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Exit policy E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_config_consistent,
    assert_exit_code_stable,
    assert_no_state_corruption,
    assert_no_traceback,
    capture_state,
)
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _assert_config_contains(h: E2EHarness, key: str, value: str) -> None:
    content = h.config_path.read_text(encoding="utf-8")
    assert f"BIJUXCLI_{key.upper()}={value}" in content


def _restart(h: E2EHarness) -> E2EHarness:
    return E2EHarness(root=h.root)


@pytest.mark.parametrize(
    "args",
    [
        ["config", "set", "NOEQUALS"],
        ["config", "set", "=value"],
        ["config", "set", "bad key=1"],
        ["config", "set", "bad=va\nl"],
        ["config", "get", "missing_key"],
        ["config", "unset", "missing_key"],
        ["config", "export", "/no/such/path/out.env"],
        ["config", "load", "/no/such/path/in.env"],
        ["config", "set", "foo=bar", "--format", "nope"],
        ["plugins", "info", "missing_plugin"],
        ["plugins", "check", "missing_plugin"],
        ["plugins", "uninstall", "missing_plugin"],
        ["plugins", "install", "bad name"],
        ["plugins", "install", "/no/such/path"],
        ["does-not-exist"],
    ],
)
def test_invalid_inputs_do_not_corrupt_state(args: list[str]) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", "guard=1"]).returncode == 0
        _assert_config_contains(h, "guard", "1")
        before = capture_state(h)

        h2 = _restart(h)
        res = h2.run(args)
        assert res.returncode != 0
        assert_no_traceback(res.stdout + res.stderr)
        assert (res.stdout + res.stderr).strip() != ""

        h3 = _restart(h2)
        res_check = h3.run(["config", "get", "guard"])
        assert res_check.returncode == 0
        assert_no_traceback(res_check.stdout + res_check.stderr)
        _assert_config_contains(h3, "guard", "1")
        assert_no_state_corruption(before, capture_state(h3))
        assert_config_consistent(h3)


@pytest.mark.parametrize(
    "name",
    [
        "broken_plugin_a",
        "broken_plugin_b",
        "broken_plugin_c",
    ],
)
def test_broken_plugin_metadata_fails_cleanly(name: str) -> None:
    with E2EHarness() as h:
        plug_dir = h.root / name
        plug_dir.mkdir(parents=True, exist_ok=True)
        (plug_dir / "plugin.py").write_text(
            "def setup():\n    return None\n", encoding="utf-8"
        )
        (plug_dir / "plugin.json").write_text("{bad json", encoding="utf-8")
        before = capture_state(h)

        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        assert_no_traceback(res.stdout + res.stderr)
        assert res.stderr.strip() != ""
        assert res.stdout.strip() == ""
        assert not (h.plugins_dir / name).exists()

        h2 = _restart(h)
        res_list = h2.run(["plugins", "list"])
        assert res_list.returncode == 0
        assert_no_traceback(res_list.stdout + res_list.stderr)
        assert_no_state_corruption(before, capture_state(h2))


@pytest.mark.parametrize(
    "name",
    [
        "missing_meta_a",
        "missing_meta_b",
    ],
)
def test_plugin_missing_metadata_fails(name: str) -> None:
    with E2EHarness() as h:
        plug_dir = h.root / name
        plug_dir.mkdir(parents=True, exist_ok=True)
        (plug_dir / "plugin.py").write_text(
            "def setup():\n    return None\n", encoding="utf-8"
        )
        before = capture_state(h)

        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        assert_no_traceback(res.stdout + res.stderr)
        assert res.stderr.strip() != ""
        assert res.stdout.strip() == ""
        assert not (h.plugins_dir / name).exists()

        h2 = _restart(h)
        res_list = h2.run(["plugins", "list"])
        assert res_list.returncode == 0
        assert_no_traceback(res_list.stdout + res_list.stderr)
        assert_no_state_corruption(before, capture_state(h2))


@pytest.mark.parametrize(
    "name",
    [
        "conflict_meta_a",
        "conflict_meta_b",
    ],
)
def test_plugin_invalid_metadata_fields_fails(name: str) -> None:
    with E2EHarness() as h:
        plug_dir = write_dummy_plugin(h.root / name, name=name)
        meta_path = plug_dir / "plugin.json"
        meta_path.write_text(
            "\n".join(
                [
                    "{",
                    '  "name": "",',
                    '  "version": "",',
                    '  "bijux_cli_version": "",',
                    '  "enabled": true',
                    "}",
                ]
            ),
            encoding="utf-8",
        )
        before = capture_state(h)

        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        assert_no_traceback(res.stdout + res.stderr)
        assert res.stderr.strip() != ""
        assert res.stdout.strip() == ""
        assert not (h.plugins_dir / name).exists()

        h2 = _restart(h)
        res_list = h2.run(["plugins", "list"])
        assert res_list.returncode == 0
        assert_no_traceback(res_list.stdout + res_list.stderr)
        assert_no_state_corruption(before, capture_state(h2))
        assert_exit_code_stable([res.returncode, res.returncode])
