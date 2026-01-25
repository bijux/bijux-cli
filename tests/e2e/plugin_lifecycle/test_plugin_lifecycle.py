# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Intent: abusive plugin lifecycle sequences.

Why E2E: filesystem + registry consistency only visible via real CLI.
Primary invariants: plugin consistency, exit stability, no corruption.
"""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.invariants import (
    assert_exit_code_stable,
    assert_no_state_corruption,
    assert_no_traceback,
    assert_plugins_consistent,
    capture_state,
)
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow, pytest.mark.plugin]


def _restart(h: E2EHarness) -> E2EHarness:
    return E2EHarness(root=h.root)


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
@pytest.mark.core
def test_plugin_install_uninstall_reinstall(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert (h.plugins_dir / name).exists()

        h2 = _restart(h)
        res = h2.run(["plugins", "uninstall", name])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert not (h2.plugins_dir / name).exists()

        h3 = _restart(h2)
        res = h3.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert (h3.plugins_dir / name).exists()
        assert_plugins_consistent(h3)


@pytest.mark.parametrize(
    "name",
    [
        "idem_plugin_a",
        "idem_plugin_b",
        "idem_plugin_c",
        "idem_plugin_d",
    ],
)
@pytest.mark.core
def test_plugin_install_is_idempotent_with_force(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h2 = _restart(h)
        res_force = h2.run(["plugins", "install", str(dummy_dir), "--force"])
        assert res_force.returncode == 0
        assert_no_traceback(res_force.stdout + res_force.stderr)

        h3 = _restart(h2)
        res_force2 = h3.run(["plugins", "install", str(dummy_dir), "--force"])
        assert res_force2.returncode == 0
        assert_no_traceback(res_force2.stdout + res_force2.stderr)
        assert (h3.plugins_dir / name).exists()
        assert_exit_code_stable([res_force.returncode, res_force2.returncode])
        assert_plugins_consistent(h3)


@pytest.mark.parametrize(
    "name",
    [
        "idem_plugin_f",
        "idem_plugin_g",
    ],
)
def test_plugin_uninstall_is_idempotent(name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        res = h.run(["plugins", "install", str(dummy_dir)])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h2 = _restart(h)
        res = h2.run(["plugins", "uninstall", name])
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)

        h3 = _restart(h2)
        res2 = h3.run(["plugins", "uninstall", name])
        assert res2.returncode == 1
        assert_no_traceback(res2.stdout + res2.stderr)
        assert not (h3.plugins_dir / name).exists()
        assert_exit_code_stable([res2.returncode, res2.returncode])
        assert_plugins_consistent(h3)


def test_reinstall_after_failed_metadata() -> None:
    with E2EHarness() as h:
        plug_dir = h.root / "bad_plugin"
        plug_dir.mkdir(parents=True, exist_ok=True)
        (plug_dir / "plugin.py").write_text(
            "def setup():\n    return None\n", encoding="utf-8"
        )
        (plug_dir / "plugin.json").write_text("{bad json", encoding="utf-8")
        before = capture_state(h)

        res = h.run(["plugins", "install", str(plug_dir)])
        assert res.returncode != 0
        assert_no_traceback(res.stdout + res.stderr)
        assert not (h.plugins_dir / "bad_plugin").exists()
        assert_no_state_corruption(before, capture_state(h))

        h2 = _restart(h)
        good_dir = write_dummy_plugin(h2.root / "bad_plugin", name="bad_plugin")
        res2 = h2.run(["plugins", "install", str(good_dir)])
        assert res2.returncode == 0
        assert_no_traceback(res2.stdout + res2.stderr)
        assert (h2.plugins_dir / "bad_plugin").exists()


def test_plugin_install_with_quiet_trace_flags() -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / "quiet_plugin", name="quiet_plugin")
        res = h.run(
            [
                "plugins",
                "install",
                str(dummy_dir),
                "--log-level",
                "trace",
                "--quiet",
            ]
        )
        assert res.returncode == 0
        assert_no_traceback(res.stdout + res.stderr)
        assert res.stdout.strip() == ""
        assert (h.plugins_dir / "quiet_plugin").exists()
        assert_plugins_consistent(h)
