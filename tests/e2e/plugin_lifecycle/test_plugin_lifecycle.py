# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin lifecycle E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


@pytest.mark.parametrize(
    "name",
    [
        "dummy_plugin_a",
        "dummy_plugin_b",
        "dummy_plugin_c",
        "dummy_plugin_d",
        "dummy_plugin_e",
    ],
)
def test_plugin_install_uninstall_reinstall(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        assert (h.plugins_dir / name).exists()

        res = h.run(["plugins", "uninstall", name])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / name).exists()

        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        assert (h.plugins_dir / name).exists()


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
        _no_traceback(res.stdout + res.stderr)
        for _ in range(2):
            res = h.run(["plugins", "install", str(dummy_dir), "--force"])
            assert res.returncode == 0
            _no_traceback(res.stdout + res.stderr)
        assert (h.plugins_dir / name).exists()

        res = h.run(["plugins", "info", name])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)


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
        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        res = h.run(["plugins", "uninstall", name])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        for _ in range(2):
            res = h.run(["plugins", "uninstall", name])
            assert res.returncode == 1
            _no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / name).exists()
