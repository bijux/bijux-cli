# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""API/CLI parity E2E tests."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from bijux_cli.api.facade import BijuxAPI
from bijux_cli.core.di import DIContainer
from bijux_cli.services.config.contracts import ConfigProtocol
from tests.e2e.harness import E2EHarness
from tests.e2e.plugins.utils import write_dummy_plugin

pytestmark = [pytest.mark.e2e, pytest.mark.slow]


def _no_traceback(text: str) -> None:
    assert "traceback" not in text.lower()


def _json_value(payload: str) -> str:
    data = json.loads(payload)
    assert isinstance(data, dict)
    return str(data["value"])


def _sync_api_env(monkeypatch: pytest.MonkeyPatch, h: E2EHarness) -> None:
    for key, value in h.env.items():
        monkeypatch.setenv(key, value)


def _register_config_helpers(api: BijuxAPI) -> None:
    def _cfg_set(key: str, value: str) -> None:
        DIContainer.current().resolve(ConfigProtocol).set(key, value)

    def _cfg_get(key: str) -> str:
        return str(DIContainer.current().resolve(ConfigProtocol).get(key))

    api.register("cfg_set", _cfg_set)
    api.register("cfg_get", _cfg_get)


def _register_plugin_helpers(api: BijuxAPI) -> None:
    def _plugin_exists(name: str) -> bool:
        plugins_dir = Path(os.environ["BIJUXCLI_PLUGINS_DIR"])
        return (plugins_dir / name).exists()

    api.register("plugin_exists", _plugin_exists)


def test_api_set_cli_get_parity(monkeypatch: pytest.MonkeyPatch) -> None:
    with E2EHarness() as h:
        _sync_api_env(monkeypatch, h)
        api = BijuxAPI()
        _register_config_helpers(api)
        _register_plugin_helpers(api)
        api.run_sync("cfg_set", "parity_key", "from_api")

        res = h.run(["config", "get", "parity_key", "--format", "json"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)
        assert _json_value(res.stdout) == "from_api"

        res2 = h.run(["config", "set", "cli_key=from_cli"])
        assert res2.returncode == 0
        _no_traceback(res2.stdout + res2.stderr)

        content = h.config_path.read_text(encoding="utf-8")
        assert "BIJUXCLI_PARITY_KEY=from_api" in content
        assert "BIJUXCLI_CLI_KEY=from_cli" in content


def test_cli_set_api_shares_config(monkeypatch: pytest.MonkeyPatch) -> None:
    with E2EHarness() as h:
        _sync_api_env(monkeypatch, h)
        api = BijuxAPI()
        _register_config_helpers(api)
        _register_plugin_helpers(api)

        res = h.run(["config", "set", "parity_key=from_cli"])
        assert res.returncode == 0
        _no_traceback(res.stdout + res.stderr)

        res2 = h.run(["config", "get", "parity_key", "--format", "json"])
        assert res2.returncode == 0
        _no_traceback(res2.stdout + res2.stderr)
        assert _json_value(res2.stdout) == "from_cli"

        api.run_sync("cfg_set", "api_key", "from_api")
        api_value = api.run_sync("cfg_get", "api_key")
        assert api_value == "from_api"
        content = h.config_path.read_text(encoding="utf-8")
        assert "BIJUXCLI_PARITY_KEY=from_cli" in content
        assert "BIJUXCLI_API_KEY=from_api" in content


def test_api_cli_plugin_lifecycle_parity(monkeypatch: pytest.MonkeyPatch) -> None:
    with E2EHarness() as h:
        _sync_api_env(monkeypatch, h)
        api = BijuxAPI()
        _register_plugin_helpers(api)

        dummy_dir = write_dummy_plugin(h.root / "parity_plugin", name="parity_plugin")
        res_install = h.run(["plugins", "install", str(dummy_dir)])
        assert res_install.returncode == 0
        _no_traceback(res_install.stdout + res_install.stderr)
        assert (h.plugins_dir / "parity_plugin").exists()

        api_plugins_dir = Path(h.env["BIJUXCLI_PLUGINS_DIR"])
        assert (api_plugins_dir / "parity_plugin").exists()

        res_uninstall = h.run(["plugins", "uninstall", "parity_plugin"])
        assert res_uninstall.returncode == 0
        _no_traceback(res_uninstall.stdout + res_uninstall.stderr)
        assert not (h.plugins_dir / "parity_plugin").exists()

        assert not (api_plugins_dir / "parity_plugin").exists()

        dummy_dir2 = write_dummy_plugin(
            h.root / "parity_plugin2", name="parity_plugin2"
        )
        res_install2 = h.run(["plugins", "install", str(dummy_dir2)])
        assert res_install2.returncode == 0
        _no_traceback(res_install2.stdout + res_install2.stderr)
        assert (h.plugins_dir / "parity_plugin2").exists()

        res_uninstall2 = h.run(["plugins", "uninstall", "parity_plugin2"])
        assert res_uninstall2.returncode == 0
        _no_traceback(res_uninstall2.stdout + res_uninstall2.stderr)
        assert not (h.plugins_dir / "parity_plugin2").exists()

        res_uninstall2b = h.run(["plugins", "uninstall", "parity_plugin2"])
        assert res_uninstall2b.returncode == 1
        _no_traceback(res_uninstall2b.stdout + res_uninstall2b.stderr)


def test_cli_reinstall_force_api_sees_plugin(monkeypatch: pytest.MonkeyPatch) -> None:
    with E2EHarness() as h:
        _sync_api_env(monkeypatch, h)
        api = BijuxAPI()
        _register_plugin_helpers(api)

        dummy_dir = write_dummy_plugin(h.root / "force_plugin", name="force_plugin")
        res_install = h.run(["plugins", "install", str(dummy_dir)])
        assert res_install.returncode == 0
        _no_traceback(res_install.stdout + res_install.stderr)

        res_force = h.run(["plugins", "install", str(dummy_dir), "--force"])
        assert res_force.returncode == 0
        _no_traceback(res_force.stdout + res_force.stderr)

        assert api.run_sync("plugin_exists", "force_plugin") is True

        res_uninstall = h.run(["plugins", "uninstall", "force_plugin"])
        assert res_uninstall.returncode == 0
        _no_traceback(res_uninstall.stdout + res_uninstall.stderr)
