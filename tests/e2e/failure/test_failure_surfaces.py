# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Failure surface E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


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
        before = (
            h.config_path.read_text(encoding="utf-8") if h.config_path.exists() else ""
        )
        res = h.run(args)
        assert res.returncode != 0
        _no_traceback(res.stdout + res.stderr)
        after = (
            h.config_path.read_text(encoding="utf-8") if h.config_path.exists() else ""
        )
        assert before == after


@pytest.mark.parametrize(
    "name",
    [
        "broken_plugin_a",
        "broken_plugin_b",
        "broken_plugin_c",
        "broken_plugin_d",
        "broken_plugin_e",
        "broken_plugin_f",
        "broken_plugin_g",
        "broken_plugin_h",
        "broken_plugin_i",
        "broken_plugin_j",
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
        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        _no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / name).exists()


@pytest.mark.parametrize(
    "name",
    [
        "missing_meta_a",
        "missing_meta_b",
        "missing_meta_c",
        "missing_meta_d",
        "missing_meta_e",
    ],
)
def test_plugin_missing_metadata_fails(name: str) -> None:
    with E2EHarness() as h:
        plug_dir = h.root / name
        plug_dir.mkdir(parents=True, exist_ok=True)
        (plug_dir / "plugin.py").write_text(
            "def setup():\n    return None\n", encoding="utf-8"
        )
        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        _no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / name).exists()


@pytest.mark.parametrize(
    "name",
    [
        "conflict_meta_a",
        "conflict_meta_b",
        "conflict_meta_c",
        "conflict_meta_d",
        "conflict_meta_e",
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
        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        _no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / name).exists()
