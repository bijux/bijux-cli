# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Idempotence E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("idemp_a", "1"),
        ("idemp_b", "2"),
        ("idemp_c", "3"),
        ("idemp_d", "4"),
        ("idemp_e", "5"),
        ("idemp_f", "6"),
        ("idemp_g", "7"),
        ("idemp_h", "8"),
        ("idemp_i", "9"),
        ("idemp_j", "10"),
    ],
)
def test_config_set_is_idempotent(key: str, value: str) -> None:
    with E2EHarness() as h:
        for _ in range(3):
            res = h.run(["config", "set", f"{key}={value}"])
            assert res.returncode == 0
            _no_traceback(res.stdout + res.stderr)
        content = h.config_path.read_text(encoding="utf-8")
        assert content.count(f"BIJUXCLI_{key.upper()}=") == 1


@pytest.mark.parametrize(
    "key",
    [
        "unset_a",
        "unset_b",
        "unset_c",
        "unset_d",
        "unset_e",
        "unset_f",
        "unset_g",
        "unset_h",
        "unset_i",
        "unset_j",
    ],
)
def test_config_unset_is_idempotent(key: str) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", f"{key}=1"]).returncode == 0
        res = h.run(["config", "unset", key])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        for _ in range(2):
            res = h.run(["config", "unset", key])
            assert res.returncode == 1
            _no_traceback(res.stdout + res.stderr)
        content = h.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{key.upper()}=" not in content


@pytest.mark.parametrize(
    "name",
    [
        "idem_plugin_a",
        "idem_plugin_b",
        "idem_plugin_c",
        "idem_plugin_d",
        "idem_plugin_e",
    ],
)
def test_plugin_install_is_idempotent_with_force(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        for _ in range(2):
            res = h.run(["plugins", "install", str(dummy_dir), "--force"])
            assert res.returncode == 0
            _no_traceback(res.stdout + res.stderr)
        assert (h.plugins_dir / name).exists()


@pytest.mark.parametrize(
    "name",
    [
        "idem_plugin_f",
        "idem_plugin_g",
        "idem_plugin_h",
        "idem_plugin_i",
        "idem_plugin_j",
    ],
)
def test_plugin_uninstall_is_idempotent(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        assert h.run(["plugins", "install", str(dummy_dir)]).returncode == 0
        res = h.run(["plugins", "uninstall", name])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        for _ in range(2):
            res = h.run(["plugins", "uninstall", name])
            assert res.returncode == 1
            _no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / name).exists()


@pytest.mark.parametrize(
    "key",
    [
        "repeat_a",
        "repeat_b",
        "repeat_c",
        "repeat_d",
        "repeat_e",
    ],
)
def test_config_get_repeat_is_stable(key: str) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", f"{key}=1"]).returncode == 0
        for _ in range(5):
            res = h.run(["config", "get", key])
            assert res.returncode == 0
            _no_traceback(res.stdout + res.stderr)
