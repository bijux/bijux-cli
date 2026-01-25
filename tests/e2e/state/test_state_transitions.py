# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""State transition E2E tests."""

from __future__ import annotations

import pytest

from tests.e2e.harness import E2EHarness
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("alpha", "1"),
        ("beta", "2"),
        ("gamma", "3"),
        ("delta", "4"),
        ("epsilon", "5"),
        ("zeta", "6"),
        ("eta", "7"),
        ("theta", "8"),
        ("iota", "9"),
        ("kappa", "10"),
    ],
)
def test_config_set_unset_set_again(key: str, value: str) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", f"{key}={value}"]).returncode == 0
        assert h.config_path.exists()
        assert f"BIJUXCLI_{key.upper()}={value}" in h.config_path.read_text(
            encoding="utf-8"
        )

        assert h.run(["config", "unset", key]).returncode == 0
        content = h.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{key.upper()}=" not in content

        assert h.run(["config", "set", f"{key}={value}"]).returncode == 0
        assert f"BIJUXCLI_{key.upper()}={value}" in h.config_path.read_text(
            encoding="utf-8"
        )


@pytest.mark.parametrize(
    ("primary", "secondary"),
    [
        ("foo", "bar"),
        ("one", "two"),
        ("left", "right"),
        ("hot", "cold"),
        ("red", "blue"),
    ],
)
def test_config_unset_preserves_other_keys(primary: str, secondary: str) -> None:
    with E2EHarness() as h:
        assert h.run(["config", "set", f"{primary}=1"]).returncode == 0
        assert h.run(["config", "set", f"{secondary}=2"]).returncode == 0
        assert h.run(["config", "unset", primary]).returncode == 0
        content = h.config_path.read_text(encoding="utf-8")
        assert f"BIJUXCLI_{primary.upper()}=" not in content
        assert f"BIJUXCLI_{secondary.upper()}=2" in content


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
        assert h.run(["plugins", "install", str(dummy_dir)]).returncode == 0
        assert (h.plugins_dir / name).exists()

        assert h.run(["plugins", "uninstall", name]).returncode == 0
        assert not (h.plugins_dir / name).exists()

        assert h.run(["plugins", "install", str(dummy_dir)]).returncode == 0
        assert (h.plugins_dir / name).exists()


@pytest.mark.parametrize(
    ("key", "name"),
    [
        ("combo1", "combo_plugin_a"),
        ("combo2", "combo_plugin_b"),
        ("combo3", "combo_plugin_c"),
        ("combo4", "combo_plugin_d"),
        ("combo5", "combo_plugin_e"),
    ],
)
def test_config_and_plugin_state_survive_sequence(key: str, name: str) -> None:
    with E2EHarness() as h:
        dummy_dir = write_dummy_plugin(h.root / name, name=name)
        assert h.run(["config", "set", f"{key}=ok"]).returncode == 0
        assert h.run(["plugins", "install", str(dummy_dir)]).returncode == 0
        assert (h.plugins_dir / name).exists()
        assert f"BIJUXCLI_{key.upper()}=ok" in h.config_path.read_text(encoding="utf-8")
        assert h.run(["plugins", "uninstall", name]).returncode == 0
        assert (h.plugins_dir / name).exists() is False
        assert f"BIJUXCLI_{key.upper()}=ok" in h.config_path.read_text(encoding="utf-8")
