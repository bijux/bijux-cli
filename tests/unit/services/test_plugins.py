# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the services plugins module."""

# pyright: reportPrivateUsage=false

from __future__ import annotations

import asyncio
import importlib
import importlib.metadata
from pathlib import Path
import sys
from typing import Any
from unittest.mock import ANY, AsyncMock, MagicMock, Mock, call, patch

import pytest

from bijux_cli.contracts import (
    ObservabilityProtocol,
    RegistryProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.exceptions import BijuxError, ServiceError
from bijux_cli.services.plugins import (
    get_plugins_dir,
    install_plugin,
    load_plugin,
    load_plugin_config,
    uninstall_plugin,
    verify_plugin_signature,
)
from bijux_cli.services.plugins.entrypoints import (
    _compatible,
    _iter_plugin_eps,
    load_entrypoints,
)
from bijux_cli.services.plugins.groups import command_group, dynamic_choices
from bijux_cli.services.plugins.hooks import CoreSpec
from bijux_cli.services.plugins.registry import Registry


@pytest.fixture
def mock_tel() -> Mock:
    """Provide a mock TelemetryProtocol instance."""
    return Mock(spec=TelemetryProtocol)


@pytest.fixture
def mock_obs() -> Mock:
    """Provide a mock ObservabilityProtocol instance."""
    return Mock(spec=ObservabilityProtocol)


@pytest.fixture
def mock_reg() -> Mock:
    """Provide a mock RegistryProtocol instance."""
    return Mock(spec=RegistryProtocol)


@pytest.fixture
def mock_di() -> Any:
    """Provide a mock dependency injector."""
    di = Mock()
    di.resolve.return_value = Mock(spec=ObservabilityProtocol)
    return di


def test_di_none() -> None:
    """Test that _di returns None when the DI container is unavailable."""
    from bijux_cli.services.plugins import _di

    with patch("bijux_cli.core.di.DIContainer.current", side_effect=Exception):
        assert _di() is None


def test_di_success(mock_di: Any) -> None:
    """Test that _di successfully returns the current DI container."""
    from bijux_cli.services.plugins import _di

    with patch("bijux_cli.core.di.DIContainer.current", return_value=mock_di):
        assert _di() == mock_di


def test_obs_none(mock_di: Any) -> None:
    """Test that _obs returns None when the observability service cannot be resolved."""
    from bijux_cli.services.plugins import _obs

    mock_di.resolve.side_effect = KeyError
    with patch("bijux_cli.services.plugins._di", return_value=mock_di):
        assert _obs() is None


def test_obs_none_no_di() -> None:
    """Test that _obs returns None when the DI container is unavailable."""
    from bijux_cli.services.plugins import _obs

    with patch("bijux_cli.services.plugins._di", return_value=None):
        assert _obs() is None


def test_obs_success(mock_di: Any, mock_obs: Mock) -> None:
    """Test that _obs successfully resolves the observability service."""
    from bijux_cli.services.plugins import _obs

    mock_di.resolve.return_value = mock_obs
    with patch("bijux_cli.services.plugins._di", return_value=mock_di):
        assert _obs() == mock_obs


def test_tel_none(mock_di: Any) -> None:
    """Test that _tel returns None when the telemetry service cannot be resolved."""
    from bijux_cli.services.plugins import _tel

    mock_di.resolve.side_effect = KeyError
    with patch("bijux_cli.services.plugins._di", return_value=mock_di):
        assert _tel() is None


def test_tel_none_no_di() -> None:
    """Test that _tel returns None when the DI container is unavailable."""
    from bijux_cli.services.plugins import _tel

    with patch("bijux_cli.services.plugins._di", return_value=None):
        assert _tel() is None


def test_tel_success(mock_di: Any, mock_tel: Mock) -> None:
    """Test that _tel successfully resolves the telemetry service."""
    from bijux_cli.services.plugins import _tel

    mock_di.resolve.return_value = mock_tel
    with patch("bijux_cli.services.plugins._di", return_value=mock_di):
        assert _tel() == mock_tel


def test_get_plugins_dir_env(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Test that the plugins directory is correctly retrieved from the environment."""
    monkeypatch.setenv("BIJUXCLI_PLUGINS_DIR", str(tmp_path / "custom"))
    assert get_plugins_dir() == tmp_path / "custom"


def test_get_plugins_dir_default(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Test that a default plugins directory is used when the env var is not set."""
    monkeypatch.delenv("BIJUXCLI_PLUGINS_DIR", raising=False)
    with patch("bijux_cli.core.paths.PLUGINS_DIR", tmp_path / "plugins"):
        assert get_plugins_dir() == (tmp_path / "plugins").resolve()


def test_get_plugins_dir_create(tmp_path: Path) -> None:
    """Test that the plugins directory is created if it does not exist."""
    dir_path = tmp_path / "plugins"
    with patch("bijux_cli.core.paths.PLUGINS_DIR", dir_path):
        assert get_plugins_dir() == dir_path.resolve()
        assert dir_path.is_dir()


def test_get_plugins_dir_exists(tmp_path: Path) -> None:
    """Test that an existing plugins directory is correctly identified."""
    dir_path = tmp_path / "plugins"
    dir_path.mkdir()
    with patch("bijux_cli.core.paths.PLUGINS_DIR", dir_path):
        assert get_plugins_dir() == dir_path.resolve()


def test_get_plugins_dir_symlink(tmp_path: Path) -> None:
    """Test that a symlinked plugins directory path is correctly resolved."""
    real_dir = tmp_path / "real"
    real_dir.mkdir()
    sym = tmp_path / "sym"
    sym.symlink_to(real_dir)
    with patch("bijux_cli.core.paths.PLUGINS_DIR", sym):
        assert get_plugins_dir() == sym


def test_get_plugins_dir_existing_file(tmp_path: Path) -> None:
    """Test that a path pointing to a file instead of a directory is handled."""
    file_path = tmp_path / "file"
    file_path.touch()
    with patch("bijux_cli.core.paths.PLUGINS_DIR", file_path):
        assert get_plugins_dir() == file_path.resolve()


def test_load_plugin_config_no_yaml() -> None:
    """Test that loading plugin config fails if PyYAML is not installed."""
    with (
        patch.dict("sys.modules", {"yaml": None}),
        pytest.raises(BijuxError, match="PyYAML"),
    ):
        load_plugin_config("test")


def test_load_plugin_config_missing(tmp_path: Path) -> None:
    """Test that loading a missing plugin config returns an empty dictionary."""
    with patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path):
        assert not load_plugin_config("missing")


def test_load_plugin_config_empty(tmp_path: Path) -> None:
    """Test that loading an empty plugin config file returns an empty dictionary."""
    cfg = tmp_path / "test" / "config.yaml"
    cfg.parent.mkdir()
    cfg.write_text("")
    with (
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
        patch("yaml.safe_load", return_value=None),
    ):
        assert not load_plugin_config("test")


def test_load_plugin_config_parse_fail(tmp_path: Path, mock_tel: Mock) -> None:
    """Ensure parse failures raise BijuxError and emit a telemetry event."""
    cfg = tmp_path / "test" / "config.yaml"
    cfg.parent.mkdir()
    cfg.touch()
    with (  # noqa: SIM117
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
        patch("yaml.safe_load", side_effect=Exception("parse fail")),
        patch("bijux_cli.services.plugins._tel", return_value=mock_tel),
    ):
        with pytest.raises(BijuxError, match="parse fail"):  # noqa: SIM117
            load_plugin_config("test")

    mock_tel.event.assert_called_with("plugin_config_failed", ANY)


def test_load_plugin_config_parse_fail_no_tel(tmp_path: Path) -> None:
    """Test that a parse failure is handled when no telemetry service is available."""
    cfg = tmp_path / "test" / "config.yaml"
    cfg.parent.mkdir()
    cfg.touch()
    with (
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
        patch("yaml.safe_load", side_effect=Exception("parse fail")),
        patch("bijux_cli.services.plugins._tel", return_value=None),
        pytest.raises(BijuxError, match="parse fail"),
    ):
        load_plugin_config("test")


def test_load_plugin_config_success(
    tmp_path: Path, mock_tel: Mock, mock_obs: Mock
) -> None:
    """Test the successful loading of a plugin configuration file."""
    cfg = tmp_path / "test" / "config.yaml"
    cfg.parent.mkdir()
    cfg.write_text("key: val")
    with (
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
        patch("yaml.safe_load", return_value={"key": "val"}),
        patch("bijux_cli.services.plugins._tel", return_value=mock_tel),
        patch("bijux_cli.services.plugins._obs", return_value=mock_obs),
    ):
        assert load_plugin_config("test") == {"key": "val"}
        mock_obs.log.assert_called_with(
            "info", "Loaded plugin config", extra={"name": "test"}
        )
        mock_tel.event.assert_called_with("plugin_config_loaded", {"name": "test"})


def test_load_plugin_config_success_no_obs(tmp_path: Path, mock_tel: Mock) -> None:
    """Test successful config loading when no observability service is available."""
    cfg = tmp_path / "test" / "config.yaml"
    cfg.parent.mkdir()
    cfg.write_text("key: val")
    with (
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
        patch("yaml.safe_load", return_value={"key": "val"}),
        patch("bijux_cli.services.plugins._tel", return_value=mock_tel),
        patch("bijux_cli.services.plugins._obs", return_value=None),
    ):
        assert load_plugin_config("test") == {"key": "val"}
        mock_tel.event.assert_called_with("plugin_config_loaded", {"name": "test"})


def test_load_plugin_config_success_no_tel(tmp_path: Path, mock_obs: Mock) -> None:
    """Test successful config loading when no telemetry service is available."""
    cfg = tmp_path / "test" / "config.yaml"
    cfg.parent.mkdir()
    cfg.write_text("key: val")
    with (
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
        patch("yaml.safe_load", return_value={"key": "val"}),
        patch("bijux_cli.services.plugins._tel", return_value=None),
        patch("bijux_cli.services.plugins._obs", return_value=mock_obs),
    ):
        assert load_plugin_config("test") == {"key": "val"}
        mock_obs.log.assert_called_with(
            "info", "Loaded plugin config", extra={"name": "test"}
        )


def test_verify_plugin_signature_no_sig(tmp_path: Path, mock_tel: Mock) -> None:
    """Test that signature verification returns False for an unsigned plugin."""
    path = tmp_path / "plugin.py"
    path.touch()
    with patch("bijux_cli.services.plugins._tel", return_value=mock_tel):
        assert not verify_plugin_signature(path, "key")
        mock_tel.event.assert_called_with("plugin_unsigned", {"path": str(path)})


def test_verify_plugin_signature_no_sig_no_tel(tmp_path: Path) -> None:
    """Test signature verification for an unsigned plugin with no telemetry."""
    path = tmp_path / "plugin.py"
    path.touch()
    with patch("bijux_cli.services.plugins._tel", return_value=None):
        assert not verify_plugin_signature(path, "key")


def test_verify_plugin_signature_no_key(tmp_path: Path) -> None:
    """Test that signature verification fails if no public key is provided."""
    path = tmp_path / "plugin.py"
    path.touch()
    sig = path.with_suffix(".py.sig")
    sig.touch()
    with pytest.raises(BijuxError, match="no public_key"):
        verify_plugin_signature(path, None)


def test_verify_plugin_signature_success(tmp_path: Path, mock_tel: Mock) -> None:
    """Test the successful verification of a plugin signature."""
    path = tmp_path / "plugin.py"
    path.touch()
    sig = path.with_suffix(".py.sig")
    sig.touch()
    with patch("bijux_cli.services.plugins._tel", return_value=mock_tel):
        assert verify_plugin_signature(path, "key")
        mock_tel.event.assert_called_with(
            "plugin_signature_verified", {"path": str(path)}
        )


def test_verify_plugin_signature_success_no_tel(tmp_path: Path) -> None:
    """Test successful signature verification with no telemetry."""
    path = tmp_path / "plugin.py"
    path.touch()
    sig = path.with_suffix(".py.sig")
    sig.touch()
    with patch("bijux_cli.services.plugins._tel", return_value=None):
        assert verify_plugin_signature(path, "key")


def test_load_plugin_missing(tmp_path: Path) -> None:
    """Test that loading a missing plugin file raises an error."""
    with pytest.raises(BijuxError, match="not found"):
        load_plugin(tmp_path / "missing.py", "mod")


def test_load_plugin_no_spec(tmp_path: Path) -> None:
    """Test that a failure to create an import spec is handled."""
    path = tmp_path / "plugin.py"
    path.touch()
    with patch(  # noqa: SIM117
        "importlib.util.spec_from_file_location", return_value=None
    ):
        with pytest.raises(BijuxError, match="Cannot import"):
            load_plugin(path, "mod")


def test_load_plugin_no_loader(tmp_path: Path) -> None:
    """Test that a missing loader in the import spec is handled."""
    path = tmp_path / "plugin.py"
    path.touch()
    spec = Mock(loader=None)

    with (
        patch("importlib.util.spec_from_file_location", return_value=spec),
        pytest.raises(BijuxError, match="Cannot import"),
    ):
        load_plugin(path, "mod")


def test_load_plugin_verify_called(tmp_path: Path) -> None:
    """Test that signature verification is called when a public key is provided."""
    path = tmp_path / "plugin.py"
    path.touch()

    with patch("bijux_cli.services.plugins.verify_plugin_signature") as mock_verify:
        with pytest.raises(BijuxError):
            load_plugin(path, "mod", public_key="key")

        mock_verify.assert_called_once_with(path, "key")


def test_load_plugin_verify_not_called(tmp_path: Path) -> None:
    """Test that signature verification is not called without a public key."""
    path = tmp_path / "plugin.py"
    path.touch()

    with patch("bijux_cli.services.plugins.verify_plugin_signature") as mock_verify:
        with pytest.raises(BijuxError):
            load_plugin(path, "mod")

        mock_verify.assert_not_called()


def test_load_plugin_exec_module(tmp_path: Path) -> None:
    """Test the successful execution and loading of a plugin module."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: pass")
    plugin = load_plugin(path, "test_mod")
    assert plugin.__class__.__name__ == "Plugin"


def test_load_plugin_no_class(tmp_path: Path) -> None:
    """Test that an error is raised if the loaded module has no 'Plugin' class."""
    path = tmp_path / "plugin.py"
    path.write_text("")
    with pytest.raises(BijuxError, match="No `Plugin` class"):
        load_plugin(path, "mod")


def test_load_plugin_version_str(tmp_path: Path) -> None:
    """Test that the plugin version is correctly cast to a string."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: version = 1")
    plugin = load_plugin(path, "mod")
    assert plugin.version == "1"


def test_load_plugin_class_version_str(tmp_path: Path) -> None:
    """Test that the plugin class's version attribute is correctly cast to a string."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: version = 1")
    plugin = load_plugin(path, "mod")
    assert plugin.__class__.version == "1"


def test_load_plugin_incompatible(tmp_path: Path) -> None:
    """Test that an incompatible CLI version requirement prevents loading."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: requires_cli_version = '>999'")
    with pytest.raises(BijuxError, match="requires CLI"):
        load_plugin(path, "mod")


def test_load_plugin_obs_warn(tmp_path: Path, mock_di: Any, mock_obs: Mock) -> None:
    """Test that a warning is logged if the plugin has no CLI attribute."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: pass")
    mock_di.resolve.return_value = mock_obs
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._tel", return_value=None),
    ):
        load_plugin(path, "mod")
        mock_obs.log.assert_called_with("warning", ANY, extra=ANY)


def test_load_plugin_no_obs_warn(tmp_path: Path, mock_di: Any) -> None:
    """Test that no warning is logged if the observability service is unavailable."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: pass")
    mock_di.resolve.side_effect = KeyError
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._tel", return_value=None),
    ):
        load_plugin(path, "mod")


def test_load_plugin_tel_loaded(tmp_path: Path, mock_di: Any, mock_tel: Mock) -> None:
    """Test that a telemetry event is sent after a plugin is loaded."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: name = 'test'")
    mock_di.resolve.return_value = mock_tel
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._obs", return_value=None),
    ):
        load_plugin(path, "mod")
        mock_tel.event.assert_called_with("plugin_loaded", {"name": "test"})


def test_load_plugin_tel_loaded_no_name(
    tmp_path: Path, mock_di: Any, mock_tel: Mock
) -> None:
    """Test that the module name is used in telemetry if the plugin has no name attribute."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: pass")
    mock_di.resolve.return_value = mock_tel
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._obs", return_value=None),
    ):
        load_plugin(path, "mod")
        mock_tel.event.assert_called_with("plugin_loaded", {"name": "mod"})


def test_load_plugin_no_tel_loaded(tmp_path: Path, mock_di: Any) -> None:
    """Test that no telemetry event is sent if the service is unavailable."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: name = 'test'")
    mock_di.resolve.side_effect = KeyError
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._obs", return_value=None),
    ):
        load_plugin(path, "mod")


def test_load_plugin_no_cli_attr(tmp_path: Path, mock_di: Any, mock_obs: Mock) -> None:
    """Test that a warning is logged if the plugin class lacks a 'cli' attribute."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: pass")
    mock_di.resolve.return_value = mock_obs
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._tel", return_value=None),
    ):
        load_plugin(path, "mod")
        mock_obs.log.assert_called_with("warning", ANY, extra=ANY)


def test_load_plugin_cli_attr_not_callable(
    tmp_path: Path, mock_di: Any, mock_obs: Mock
) -> None:
    """Test that a warning is logged if the plugin's 'cli' attribute is not callable."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin: cli = 1")
    mock_di.resolve.return_value = mock_obs
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._tel", return_value=None),
    ):
        load_plugin(path, "mod")
        mock_obs.log.assert_called_with("warning", ANY, extra=ANY)


def test_load_plugin_cli_attr_callable(
    tmp_path: Path, mock_di: Any, mock_obs: Mock
) -> None:
    """Test that no warning is logged if the plugin's 'cli' attribute is callable."""
    path = tmp_path / "plugin.py"
    path.write_text("class Plugin:\n    def cli(self):\n        pass")
    mock_di.resolve.return_value = mock_obs
    with (
        patch("bijux_cli.services.plugins._di", return_value=mock_di),
        patch("bijux_cli.services.plugins._tel", return_value=None),
    ):
        load_plugin(path, "mod")
        mock_obs.log.assert_not_called()


def test_load_plugin_exec_fail(tmp_path: Path) -> None:
    """Test that an exception during module execution is wrapped in a BijuxError."""
    path = tmp_path / "plugin.py"
    path.write_text("raise Exception('fail')")
    with pytest.raises(BijuxError, match="fail"):
        load_plugin(path, "mod")


def test_load_plugin_module_removed_on_fail(tmp_path: Path) -> None:
    """Test that a failed module is removed from sys.modules."""
    path = tmp_path / "plugin.py"
    path.write_text("raise Exception('fail')")
    module_name = "test_fail_mod"
    with pytest.raises(BijuxError):
        load_plugin(path, module_name)
    assert module_name not in sys.modules


def test_uninstall_plugin_not_found(mock_reg: Mock, mock_tel: Mock) -> None:
    """Test that uninstalling a non-existent plugin returns False."""
    mock_reg.has.return_value = False
    with patch("bijux_cli.services.plugins._tel", return_value=mock_tel):
        assert not uninstall_plugin("missing", mock_reg)
        mock_tel.event.assert_called_with(
            "plugin_uninstall_not_found", {"name": "missing"}
        )


def test_uninstall_plugin_not_found_no_tel(mock_reg: Mock) -> None:
    """Test uninstalling a non-existent plugin with no telemetry service."""
    mock_reg.has.return_value = False
    with patch("bijux_cli.services.plugins._tel", return_value=None):
        assert not uninstall_plugin("missing", mock_reg)


def test_uninstall_plugin_success(
    tmp_path: Path, mock_reg: Mock, mock_tel: Mock
) -> None:
    """Test the successful uninstallation of a plugin."""
    plug_dir = tmp_path / "test"
    plug_dir.mkdir()
    mock_reg.has.return_value = True
    with (
        patch("bijux_cli.services.plugins._tel", return_value=mock_tel),
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
    ):
        assert uninstall_plugin("test", mock_reg)
        assert not plug_dir.exists()
        mock_reg.deregister.assert_called_with("test")
        mock_tel.event.assert_called_with("plugin_uninstalled", {"name": "test"})


def test_uninstall_plugin_success_no_tel(tmp_path: Path, mock_reg: Mock) -> None:
    """Test successful uninstallation with no telemetry service."""
    plug_dir = tmp_path / "test"
    plug_dir.mkdir()
    mock_reg.has.return_value = True
    with (
        patch("bijux_cli.services.plugins._tel", return_value=None),
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
    ):
        assert uninstall_plugin("test", mock_reg)
        assert not plug_dir.exists()
        mock_reg.deregister.assert_called_with("test")


def test_uninstall_plugin_rmtree_ignore_errors(
    tmp_path: Path, mock_reg: Mock, mock_tel: Mock
) -> None:
    """Test that errors during directory removal are ignored."""
    plug_dir = tmp_path / "test"
    plug_dir.mkdir()
    mock_reg.has.return_value = True
    with (
        patch("shutil.rmtree") as mock_rmtree,
        patch("bijux_cli.services.plugins._tel", return_value=mock_tel),
        patch("bijux_cli.services.plugins.get_plugins_dir", return_value=tmp_path),
    ):
        mock_rmtree.side_effect = Exception("error")
        uninstall_plugin("test", mock_reg)
        mock_rmtree.assert_called_with(plug_dir, ignore_errors=True)


def test_install_plugin() -> None:
    """Test that the install_plugin function is not implemented."""
    with pytest.raises(NotImplementedError):
        install_plugin()


def test_lazy_import() -> None:
    """Test the lazy loading of submodules via __getattr__."""
    import bijux_cli.services.plugins as plugins

    _ = plugins.hooks
    _ = plugins.entrypoints
    _ = plugins.groups
    _ = plugins.registry
    _ = plugins.command_group
    _ = plugins.dynamic_choices
    _ = plugins.load_entrypoints
    with pytest.raises(AttributeError):
        _ = plugins.invalid


def test_iter_plugin_eps_success() -> None:
    """Test the successful iteration of plugin entry points."""
    mock_ep = Mock()
    mock_eps = Mock(select=Mock(return_value=[mock_ep]))
    with patch("importlib.metadata.entry_points", return_value=mock_eps):
        assert _iter_plugin_eps() == [mock_ep]


def test_iter_plugin_eps_fail() -> None:
    """Test that an exception during entry point iteration is handled."""
    with patch("importlib.metadata.entry_points", side_effect=Exception):
        assert not _iter_plugin_eps()


def test_compatible_true() -> None:
    """Test that a compatible plugin API version is correctly identified."""
    plugin = Mock(requires_api_version=">=1.0")
    with patch("bijux_cli.api_version", "1.0"):
        assert _compatible(plugin)


def test_compatible_false() -> None:
    """Test that an incompatible plugin API version is correctly identified."""
    plugin = Mock(requires_api_version=">2.0")
    with patch("bijux_cli.api_version", "1.0"):
        assert not _compatible(plugin)


def test_compatible_no_attr() -> None:
    """Test that a plugin without a version requirement is considered compatible."""
    plugin = Mock(spec=[])
    with patch("bijux_cli.api_version", "1.0"):
        assert _compatible(plugin)


def test_compatible_parse_fail_spec() -> None:
    """Test that an invalid version specifier is handled."""
    plugin = Mock(requires_api_version="invalid")
    with (
        patch("bijux_cli.api_version", "1.0"),
        patch("packaging.specifiers.SpecifierSet", side_effect=ValueError),
    ):
        assert not _compatible(plugin)


def test_compatible_parse_fail_version() -> None:
    """Test that an invalid CLI API version is handled."""
    plugin = Mock(requires_api_version=">=1.0")
    with patch("bijux_cli.api_version", "invalid"):
        assert not _compatible(plugin)


def test_compatible_contains_fail() -> None:
    """Test that an error during version comparison is handled."""
    plugin = Mock(requires_api_version=">=1.0")
    with (
        patch("bijux_cli.api_version", "1.0"),
        patch("packaging.specifiers.SpecifierSet.contains", side_effect=ValueError),
    ):
        assert not _compatible(plugin)


@pytest.mark.asyncio
async def test_load_entrypoints_success(mock_di: Any, mock_reg: Mock) -> None:
    """Test the successful loading of plugin entry points."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">=1.0"
    mock_plugin.startup = AsyncMock()
    mock_plugin_class.return_value = mock_plugin
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [mock_obs, mock_tel]
    with patch(  # noqa: SIM117
        "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
        return_value=[mock_ep],
    ):
        with patch(
            "bijux_cli.services.plugins.entrypoints._compatible", return_value=True
        ):
            await load_entrypoints(di=mock_di, registry=mock_reg)
            assert mock_plugin_class.version == "1"
            assert mock_plugin.version == "1"
            mock_reg.register.assert_called_with("test", mock_plugin, version="1")
            mock_plugin.startup.assert_awaited_with(mock_di)
            mock_obs.log.assert_called_with("info", "Loaded plugin 'test'")
            mock_tel.event.assert_called_with(
                "entrypoint_plugin_loaded", {"name": "test"}
            )


@pytest.mark.asyncio
async def test_load_entrypoints_success_sync_startup(
    mock_di: Any, mock_reg: Mock
) -> None:
    """Test loading an entry point with a synchronous startup hook."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">=1.0"
    mock_plugin.startup = Mock()
    mock_plugin_class.return_value = mock_plugin
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [mock_obs, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_plugin.startup.assert_called_with(mock_di)


@pytest.mark.asyncio
async def test_load_entrypoints_success_no_startup(
    mock_di: Any, mock_reg: Mock
) -> None:
    """Test loading an entry point with no startup hook."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">=1.0"
    mock_plugin_class.return_value = mock_plugin
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [mock_obs, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)


@pytest.mark.asyncio
async def test_load_entrypoints_success_no_obs(mock_di: Any, mock_reg: Mock) -> None:
    """Test loading entry points with no observability service."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">=1.0"
    mock_plugin.startup = AsyncMock()
    mock_plugin_class.return_value = mock_plugin
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [None, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_tel.event.assert_called_with("entrypoint_plugin_loaded", {"name": "test"})


@pytest.mark.asyncio
async def test_load_entrypoints_success_no_tel(mock_di: Any, mock_reg: Mock) -> None:
    """Test loading entry points with no telemetry service."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">=1.0"
    mock_plugin.startup = AsyncMock()
    mock_plugin_class.return_value = mock_plugin
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_di.resolve.side_effect = [mock_obs, None]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_obs.log.assert_called_with("info", "Loaded plugin 'test'")


@pytest.mark.asyncio
async def test_load_entrypoints_incompatible(mock_di: Any, mock_reg: Mock) -> None:
    """Test that an incompatible plugin is not loaded."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">999"
    mock_plugin_class.return_value = mock_plugin
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [mock_obs, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=False),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        assert mock_plugin_class.version == 1
        assert mock_plugin.version == 1
        mock_reg.register.assert_not_called()
        mock_obs.log.assert_called_with("error", ANY, extra=ANY)
        mock_tel.event.assert_called_with("entrypoint_plugin_failed", ANY)


@pytest.mark.asyncio
async def test_load_entrypoints_incompatible_no_obs(
    mock_di: Any, mock_reg: Mock
) -> None:
    """Test handling an incompatible plugin with no observability service."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">999"
    mock_plugin_class.return_value = mock_plugin
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [None, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=False),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_tel.event.assert_called_with("entrypoint_plugin_failed", ANY)


@pytest.mark.asyncio
async def test_load_entrypoints_incompatible_no_tel(
    mock_di: Any, mock_reg: Mock
) -> None:
    """Test handling an incompatible plugin with no telemetry service."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_plugin_class = Mock()
    mock_plugin_class.version = 1
    mock_ep.load.return_value = mock_plugin_class
    mock_plugin = Mock()
    mock_plugin.version = 1
    mock_plugin.requires_api_version = ">999"
    mock_plugin_class.return_value = mock_plugin
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_di.resolve.side_effect = [mock_obs, None]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=False),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_obs.log.assert_called_with("error", ANY, extra=ANY)


@pytest.mark.asyncio
async def test_load_entrypoints_fail(mock_di: Any, mock_reg: Mock) -> None:
    """Test that a failure during entry point loading is handled."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_ep.load = Mock(side_effect=Exception("load fail"))
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [mock_obs, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_reg.deregister.assert_called_with("test")
        mock_obs.log.assert_called_with("error", ANY, extra=ANY)
        mock_tel.event.assert_called_with("entrypoint_plugin_failed", ANY)


@pytest.mark.asyncio
async def test_load_entrypoints_fail_no_obs(mock_di: Any, mock_reg: Mock) -> None:
    """Test handling a load failure with no observability service."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_ep.load = Mock(side_effect=Exception("load fail"))
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [None, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_tel.event.assert_called_with("entrypoint_plugin_failed", ANY)


@pytest.mark.asyncio
async def test_load_entrypoints_fail_no_tel(mock_di: Any, mock_reg: Mock) -> None:
    """Test handling a load failure with no telemetry service."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_ep.load = Mock(side_effect=Exception("load fail"))
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_di.resolve.side_effect = [mock_obs, None]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_obs.log.assert_called_with("error", ANY, extra=ANY)


@pytest.mark.asyncio
async def test_load_entrypoints_deregister_suppress(
    mock_di: Any, mock_reg: Mock
) -> None:
    """Test that an error during deregistration is suppressed."""
    mock_ep = Mock()
    mock_ep.name = "test"
    mock_ep.load = Mock(side_effect=Exception("load fail"))
    mock_reg.deregister.side_effect = Exception("deregister fail")
    mock_obs = Mock(spec=ObservabilityProtocol)
    mock_tel = Mock(spec=TelemetryProtocol)
    mock_di.resolve.side_effect = [mock_obs, mock_tel]
    with (
        patch(
            "bijux_cli.services.plugins.entrypoints._iter_plugin_eps",
            return_value=[mock_ep],
        ),
        patch("bijux_cli.services.plugins.entrypoints._compatible", return_value=True),
    ):
        await load_entrypoints(di=mock_di, registry=mock_reg)
        mock_reg.deregister.assert_called_with("test")


def test_command_group_invalid_sub() -> None:
    """Test that creating a subcommand with an invalid name raises an error."""
    with pytest.raises(ValueError, match="spaces"):
        command_group("group")("sub cmd")


def test_command_group_register(
    mock_di: Any, mock_reg: MagicMock, mock_obs: MagicMock, mock_tel: MagicMock
) -> None:
    """Test the successful registration of a command group and subcommand."""
    mock_di.resolve.side_effect = [mock_reg, mock_obs, mock_tel]
    with patch("bijux_cli.core.di.DIContainer.current", return_value=mock_di):
        grp = command_group("test", version="1.0")
        sub = grp("sub")

        @sub
        def func() -> None:
            pass

        mock_reg.register.assert_called_with("test sub", func, version="1.0")
        mock_obs.log.assert_called_with("info", "Registered command group", extra=ANY)
        mock_tel.event.assert_called_with("command_group_registered", ANY)


def test_command_group_no_obs(mock_di: Any, mock_reg: Mock, mock_tel: Mock) -> None:
    """Test command group registration with no observability service."""
    mock_di.resolve.side_effect = [mock_reg, KeyError, mock_tel]
    with patch("bijux_cli.core.di.DIContainer.current", return_value=mock_di):
        grp = command_group("test")
        sub = grp("sub")

        @sub
        def func() -> None:  # pyright: ignore[reportUnusedFunction]
            pass

        mock_reg.register.assert_called()
        mock_tel.event.assert_called()


def test_command_group_no_tel(mock_di: Any, mock_reg: Mock, mock_obs: Mock) -> None:
    """Test command group registration with no telemetry service."""
    mock_di.resolve.side_effect = [mock_reg, mock_obs, KeyError]
    with patch("bijux_cli.core.di.DIContainer.current", return_value=mock_di):
        grp = command_group("test")
        sub = grp("sub")

        @sub
        def func() -> None:  # pyright: ignore[reportUnusedFunction]
            pass

        mock_reg.register.assert_called()
        mock_obs.log.assert_called()


def test_command_group_no_di() -> None:
    """Test that command group registration fails if the DI container is unavailable."""
    with patch("bijux_cli.core.di.DIContainer.current", side_effect=KeyError):
        grp = command_group("test")
        sub = grp("sub")
        with pytest.raises(RuntimeError):

            @sub
            def func() -> None:  # pyright: ignore[reportUnusedFunction]
                pass


def test_dynamic_choices_case_sensitive() -> None:
    """Test case-sensitive dynamic choices for command completion."""

    def cb() -> list[str]:
        """Provide a list of choices."""
        return ["abc", "abd", "bcd"]

    completer = dynamic_choices(cb)
    assert completer(None, None, "ab") == ["abc", "abd"]  # type: ignore[arg-type]
    assert not completer(None, None, "Ab")  # type: ignore[arg-type]


def test_dynamic_choices_case_insensitive() -> None:
    """Test case-insensitive dynamic choices for command completion."""

    def cb() -> list[str]:
        """Provide a list of choices."""
        return ["abc", "abd", "bcd"]

    completer = dynamic_choices(cb, case_sensitive=False)
    assert completer(None, None, "ab") == ["abc", "abd"]  # type: ignore[arg-type]
    assert completer(None, None, "Ab") == ["abc", "abd"]  # type: ignore[arg-type]


def test_core_spec_init(mock_di: Any) -> None:
    """Test the initialization of the CoreSpec hook specification."""
    CoreSpec(mock_di)


@pytest.mark.asyncio
async def test_core_spec_startup(mock_di: Any) -> None:
    """Test the startup hook of the CoreSpec."""
    spec = CoreSpec(mock_di)
    with patch.object(spec._log, "log") as mock_log:
        await spec.startup()
        mock_log.assert_called_with("debug", "Hook startup called", extra={})


@pytest.mark.asyncio
async def test_core_spec_shutdown(mock_di: Any) -> None:
    """Test the shutdown hook of the CoreSpec."""
    spec = CoreSpec(mock_di)
    with patch.object(spec._log, "log") as mock_log:
        await spec.shutdown()
        mock_log.assert_called_with("debug", "Hook shutdown called", extra={})


@pytest.mark.asyncio
async def test_core_spec_pre_execute(mock_di: Any) -> None:
    """Test the pre_execute hook of the CoreSpec."""
    spec = CoreSpec(mock_di)
    with patch.object(spec._log, "log") as mock_log:
        await spec.pre_execute("cmd", (1,), {"k": "v"})
        mock_log.assert_called_with("debug", "Hook pre_execute called", extra=ANY)


@pytest.mark.asyncio
async def test_core_spec_post_execute(mock_di: Any) -> None:
    """Test the post_execute hook of the CoreSpec."""
    spec = CoreSpec(mock_di)
    with patch.object(spec._log, "log") as mock_log:
        await spec.post_execute("cmd", "result")
        mock_log.assert_called_with("debug", "Hook post_execute called", extra=ANY)


def test_core_spec_health(mock_di: Any) -> None:
    """Test the health hook of the CoreSpec."""
    spec = CoreSpec(mock_di)
    with patch.object(spec._log, "log") as mock_log:
        assert spec.health() is True
        mock_log.assert_called_with("debug", "Hook health called", extra={})


def test_registry_init(mock_tel: Mock) -> None:
    """Test the initialization of the Registry."""
    reg = Registry(mock_tel)
    assert not reg.mapping


def test_registry_register_success(mock_tel: Mock) -> None:
    """Test the successful registration of a plugin."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin, alias="alias", version="1.0")
    assert "test" in reg._plugins
    assert reg._plugins["test"] == plugin
    assert reg._aliases["alias"] == "test"
    assert reg._meta["test"] == {"version": "1.0"}
    mock_tel.event.assert_called_with("registry_plugin_registered", ANY)


def test_registry_register_no_alias(mock_tel: Mock) -> None:
    """Test plugin registration without an alias."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin)
    assert "test" in reg._plugins


def test_registry_register_no_version(mock_tel: Mock) -> None:
    """Test plugin registration without a version."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin)
    assert reg._meta["test"] == {"version": "unknown"}


def test_registry_register_dup_name(mock_tel: Mock) -> None:
    """Test that registering a duplicate plugin name raises an error."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin)
    with pytest.raises(ServiceError):
        reg.register("test", Mock())


def test_registry_register_dup_obj(mock_tel: Mock) -> None:
    """Test that registering the same plugin object under two names raises an error."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin)
    with pytest.raises(ServiceError):
        reg.register("test2", plugin)


def test_registry_register_dup_alias(mock_tel: Mock) -> None:
    """Test that registering a duplicate alias raises an error."""
    reg = Registry(mock_tel)
    reg.register("test", Mock(), alias="alias")
    with pytest.raises(ServiceError):
        reg.register("test2", Mock(), alias="alias")


def test_registry_register_dup_alias_name(mock_tel: Mock) -> None:
    """Test that registering an alias that conflicts with a name raises an error."""
    reg = Registry(mock_tel)
    reg.register("alias", Mock())
    with pytest.raises(ServiceError):
        reg.register("test2", Mock(), alias="alias")


def test_registry_register_pluggy_fail(mock_tel: Mock) -> None:
    """Test that a failure in the underlying pluggy manager is handled."""
    reg = Registry(mock_tel)
    with patch.object(  # noqa: SIM117
        reg._pm, "register", side_effect=ValueError("fail")
    ):
        with pytest.raises(ServiceError):
            reg.register("test", Mock())


def test_registry_register_tel_fail(mock_tel: Mock) -> None:
    """Test that a telemetry failure during registration is handled."""
    mock_tel.event.side_effect = [RuntimeError, None]
    reg = Registry(mock_tel)
    reg.register("test", Mock())
    mock_tel.event.assert_any_call("registry_plugin_registered", ANY)
    mock_tel.event.assert_any_call("registry_telemetry_failed", ANY)


def test_registry_deregister_success(mock_tel: Mock) -> None:
    """Test the successful deregistration of a plugin."""
    reg = Registry(mock_tel)
    reg.register("test", Mock(), alias="alias")
    reg.deregister("test")
    assert "test" not in reg._plugins
    assert "alias" not in reg._aliases
    mock_tel.event.assert_any_call("registry_plugin_deregistered", {"name": "test"})


def test_registry_deregister_alias(mock_tel: Mock) -> None:
    """Test deregistering a plugin via its alias."""
    reg = Registry(mock_tel)
    reg.register("test", Mock(), alias="alias")
    reg.deregister("alias")
    assert "test" not in reg._plugins


def test_registry_deregister_not_found(mock_tel: Mock) -> None:
    """Test that deregistering a non-existent plugin is a no-op."""
    reg = Registry(mock_tel)
    reg.deregister("missing")


def test_registry_deregister_pluggy_fail(mock_tel: Mock) -> None:
    """Test that a failure in the underlying pluggy manager is handled."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin)
    with patch.object(  # noqa: SIM117
        reg._pm, "unregister", side_effect=ValueError("fail")
    ):
        with pytest.raises(ServiceError):
            reg.deregister("test")


def test_registry_deregister_tel_fail(mock_tel: Mock) -> None:
    """Test that a telemetry failure during deregistration is handled."""
    mock_tel.event.side_effect = [None, RuntimeError, None]
    reg = Registry(mock_tel)
    reg.register("test", Mock())
    reg.deregister("test")
    mock_tel.event.assert_any_call("registry_telemetry_failed", ANY)


def test_registry_get_success(mock_tel: Mock) -> None:
    """Test the successful retrieval of a plugin."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin)
    assert reg.get("test") == plugin
    mock_tel.event.assert_any_call("registry_plugin_retrieved", {"name": "test"})


def test_registry_get_alias(mock_tel: Mock) -> None:
    """Test retrieving a plugin via its alias."""
    reg = Registry(mock_tel)
    plugin = Mock()
    reg.register("test", plugin, alias="alias")
    assert reg.get("alias") == plugin


def test_registry_get_missing(mock_tel: Mock) -> None:
    """Test that retrieving a non-existent plugin raises an error."""
    reg = Registry(mock_tel)
    with pytest.raises(ServiceError, match="not found"):
        reg.get("missing")
    mock_tel.event.assert_any_call("registry_plugin_retrieve_failed", ANY)


def test_registry_get_tel_fail(mock_tel: Mock) -> None:
    """Test that a telemetry failure during retrieval is handled."""
    mock_tel.event.side_effect = [None, RuntimeError, None]
    reg = Registry(mock_tel)
    reg.register("test", Mock())
    reg.get("test")
    mock_tel.event.assert_any_call("registry_telemetry_failed", ANY)


def test_registry_get_retrieve_failed_tel_fail(mock_tel: Mock) -> None:
    """Test telemetry failure handling when a plugin retrieval fails."""
    mock_tel.event.side_effect = [RuntimeError, None]
    reg = Registry(mock_tel)
    with pytest.raises(ServiceError):
        reg.get("missing")
    mock_tel.event.assert_any_call("registry_telemetry_failed", ANY)


def test_registry_names(mock_tel: Mock) -> None:
    """Test retrieving the list of all registered plugin names."""
    reg = Registry(mock_tel)
    reg.register("a", Mock())
    reg.register("b", Mock())
    assert sorted(reg.names()) == ["a", "b"]
    mock_tel.event.assert_any_call("registry_list", {"names": ANY})


def test_registry_names_tel_fail(mock_tel: Mock) -> None:
    """Test telemetry failure handling when listing names."""
    mock_tel.event.side_effect = [RuntimeError, None]
    reg = Registry(mock_tel)
    reg.names()
    mock_tel.event.assert_called_with("registry_telemetry_failed", ANY)


def test_registry_has_true(mock_tel: Mock) -> None:
    """Test the 'has' method for an existing plugin."""
    reg = Registry(mock_tel)
    reg.register("test", Mock())
    assert reg.has("test")
    mock_tel.event.assert_called_with(
        "registry_contains", {"name": "test", "result": True}
    )


def test_registry_has_alias(mock_tel: Mock) -> None:
    """Test the 'has' method for an existing alias."""
    reg = Registry(mock_tel)
    reg.register("test", Mock(), alias="alias")
    assert reg.has("alias")


def test_registry_has_false(mock_tel: Mock) -> None:
    """Test the 'has' method for a non-existent plugin."""
    reg = Registry(mock_tel)
    assert not reg.has("missing")
    mock_tel.event.assert_called_with(
        "registry_contains", {"name": "missing", "result": False}
    )


def test_registry_has_tel_fail(mock_tel: Mock) -> None:
    """Test telemetry failure handling for the 'has' method."""
    mock_tel.event.side_effect = [RuntimeError, None]
    reg = Registry(mock_tel)
    reg.has("test")
    mock_tel.event.assert_called_with("registry_telemetry_failed", ANY)


def test_registry_meta_success(mock_tel: Mock) -> None:
    """Test retrieving metadata for an existing plugin."""
    reg = Registry(mock_tel)
    reg.register("test", Mock(), version="1.0")
    assert reg.meta("test") == {"version": "1.0"}
    mock_tel.event.assert_called_with("registry_meta_retrieved", {"name": "test"})


def test_registry_meta_alias(mock_tel: Mock) -> None:
    """Test retrieving metadata via a plugin's alias."""
    reg = Registry(mock_tel)
    reg.register("test", Mock(), alias="alias", version="1.0")
    assert reg.meta("alias") == {"version": "1.0"}


def test_registry_meta_missing(mock_tel: Mock) -> None:
    """Test that retrieving metadata for a non-existent plugin returns an empty dict."""
    reg = Registry(mock_tel)
    assert not reg.meta("missing")
    mock_tel.event.assert_called_with("registry_meta_retrieved", {"name": "missing"})


def test_registry_meta_tel_fail(mock_tel: Mock) -> None:
    """Test telemetry failure handling for the 'meta' method."""
    mock_tel.event.side_effect = [RuntimeError, None]
    reg = Registry(mock_tel)
    reg.meta("test")
    mock_tel.event.assert_called_with("registry_telemetry_failed", ANY)


@pytest.mark.asyncio
async def test_registry_call_hook_sync(mock_tel: Mock) -> None:
    """Test calling a synchronous hook."""
    reg = Registry(mock_tel)
    mock_hook = Mock(return_value=[1, 2])
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        results = await reg.call_hook("test_hook", a=1)
        assert results == [1, 2]
        mock_tel.event.assert_called_with("registry_hook_called", {"hook": "test_hook"})


@pytest.mark.asyncio
async def test_registry_call_hook_sync_none(mock_tel: Mock) -> None:
    """Test a synchronous hook that returns None."""
    reg = Registry(mock_tel)
    mock_hook = Mock(return_value=[None])
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        results = await reg.call_hook("test_hook")
        assert not results


@pytest.mark.asyncio
async def test_registry_call_hook_sync_coroutine(mock_tel: Mock) -> None:
    """Test a synchronous hook that returns a coroutine."""
    reg = Registry(mock_tel)

    async def coro() -> int:
        return 42

    mock_hook = Mock(return_value=[coro()])
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        results = await reg.call_hook("test_hook")
        assert results == [42]


@pytest.mark.asyncio
async def test_registry_call_hook_async(mock_tel: Mock) -> None:
    """Test calling an asynchronous hook (async generator)."""

    async def async_gen() -> Any:
        yield await asyncio.sleep(0, 3)
        yield 4

    reg = Registry(mock_tel)
    mock_hook = Mock(return_value=async_gen())
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        results = await reg.call_hook("test_hook")
        assert results == [3, 4]


@pytest.mark.asyncio
async def test_registry_call_hook_async_none(mock_tel: Mock) -> None:
    """Test an asynchronous hook that yields None."""

    async def async_gen() -> Any:
        yield None

    reg = Registry(mock_tel)
    mock_hook = Mock(return_value=async_gen())
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        results = await reg.call_hook("test_hook")
        assert not results


@pytest.mark.asyncio
async def test_registry_call_hook_async_coroutine(mock_tel: Mock) -> None:
    """Test an asynchronous hook that yields a coroutine."""

    async def coro() -> int:
        return 42

    async def async_gen() -> Any:
        yield coro()

    reg = Registry(mock_tel)
    mock_hook = Mock(return_value=async_gen())
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        results = await reg.call_hook("test_hook")
        assert results == [42]


@pytest.mark.asyncio
async def test_registry_call_hook_no_hook(mock_tel: Mock) -> None:
    """Test that calling a non-existent hook raises an error."""
    reg = Registry(mock_tel)
    with pytest.raises(ServiceError, match="not found"):
        await reg.call_hook("missing")


@pytest.mark.asyncio
async def test_registry_call_hook_tel_fail(mock_tel: Mock) -> None:
    """Test telemetry failure handling when calling a hook."""
    mock_tel.event.side_effect = [RuntimeError, None]
    reg = Registry(mock_tel)
    mock_hook = Mock(return_value=[])
    with patch.object(reg._pm.hook, "test_hook", mock_hook, create=True):
        await reg.call_hook("test_hook")
        mock_tel.event.assert_called_with("registry_telemetry_failed", ANY)


class _DummyEP:
    """A minimal mock for importlib.metadata.EntryPoint."""

    def __init__(self, name: str, plugin_cls: Any) -> None:
        """Initialize the mock entry point."""
        self.name = name
        self._cls = plugin_cls

    def load(self) -> Any:
        """Load the mock entry point."""
        return self._cls


class _DummyEPs:
    """A mock container for entry points with a select() method."""

    def __init__(self, eps: list[Any]) -> None:
        """Initialize the mock entry point container."""
        self._eps = eps

    def select(self, *, group: str) -> list[Any]:
        """Select entry points by group."""
        assert group == "bijux_cli.plugins"
        return self._eps


@pytest.mark.asyncio
async def test_entrypoints_pkgversion_and_sync_startup(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test various branches in the entry point loading logic."""
    from packaging.version import Version as PkgVersion

    startup_calls: list[Any] = []

    class Plugin:
        version = "2.0"
        requires_api_version = ">=0.0.0"

        def startup(self, di: Any) -> None:
            startup_calls.append(di)

    dummy_ep = _DummyEP("dummy_plugin", Plugin)
    monkeypatch.setattr(
        importlib.metadata, "entry_points", lambda: _DummyEPs([dummy_ep])
    )
    import bijux_cli

    monkeypatch.setattr(bijux_cli, "api_version", PkgVersion("0.1.0"), raising=False)

    registry = Mock(spec=RegistryProtocol)
    obs = Mock(spec=ObservabilityProtocol)
    tel = Mock(spec=TelemetryProtocol)

    class _DI:
        def resolve(self, token: Any, default: Any = None) -> Any:
            if token is RegistryProtocol:
                return registry
            if token is ObservabilityProtocol:
                return obs
            if token is TelemetryProtocol:
                return tel
            return default

    di = _DI()
    await load_entrypoints(di=di, registry=registry)  # type: ignore[arg-type]
    assert startup_calls
    assert startup_calls[0] is di
    registry.register.assert_called_once()
    reg_name, reg_plugin = registry.register.call_args.args[:2]
    assert reg_name == "dummy_plugin"
    assert registry.register.call_args.kwargs.get("version") == "2.0"
    obs.log.assert_any_call("info", "Loaded plugin 'dummy_plugin'")
    assert any(
        c == call("entrypoint_plugin_loaded", {"name": "dummy_plugin"})
        for c in tel.event.call_args_list
    )


def test_plugins_dunder_getattr_imports_submodule() -> None:
    """Test that __getattr__ correctly lazy-loads submodules."""
    import bijux_cli.services.plugins as plugins

    importlib.reload(plugins)
    hooks_mod = plugins.hooks
    assert hasattr(hooks_mod, "CoreSpec")


class _EP:
    """A minimal mock for importlib.metadata.EntryPoint."""

    def __init__(self, name: str, cls: Any) -> None:
        """Initialize the mock entry point."""
        self.name, self._cls = name, cls

    def load(self) -> Any:
        """Load the mock entry point."""
        return self._cls


class _EPs:
    """A mock container for entry points."""

    def __init__(self, eps: list[Any]) -> None:
        """Initialize the mock entry point container."""
        self._eps = eps

    def select(self, *, group: str) -> list[Any]:
        """Select entry points by group."""
        return self._eps


@pytest.mark.asyncio
async def test_entrypoints_startup_not_callable(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Test loading an entry point where the startup attribute is not callable."""

    class Plugin:
        version = "1.0"
        requires_api_version = ">=0.0.0"

    ep = _EP("noop_plugin", Plugin)
    monkeypatch.setattr(
        __import__("importlib.metadata", fromlist=["entry_points"]),
        "entry_points",
        lambda: _EPs([ep]),
    )

    import bijux_cli

    monkeypatch.setattr(bijux_cli, "api_version", "1.0", raising=False)

    registry = Mock(spec=RegistryProtocol)
    obs = Mock(spec=ObservabilityProtocol)
    tel = Mock(spec=TelemetryProtocol)

    class DI:
        def resolve(self, token: Any, default: Any = None) -> Any:
            if token is RegistryProtocol:
                return registry
            if token is ObservabilityProtocol:
                return obs
            if token is TelemetryProtocol:
                return tel
            return default

    await load_entrypoints(di=DI(), registry=registry)  # type: ignore[arg-type]


def test_dunder_getattr_submodule_hooks(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test lazy loading of the 'hooks' submodule."""
    import bijux_cli.services.plugins as plugins

    if hasattr(plugins, "hooks"):
        del sys.modules[plugins.__name__].hooks  # pyright: ignore[reportAttributeAccessIssue]
    hooks_mod = plugins.hooks
    assert hooks_mod.__name__.endswith("hooks")
    assert plugins.hooks is hooks_mod


def test_dunder_getattr_submodule_entrypoints(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test lazy loading of the 'entrypoints' submodule."""
    import bijux_cli.services.plugins as plugins

    if hasattr(plugins, "entrypoints"):
        del sys.modules[plugins.__name__].entrypoints  # pyright: ignore[reportAttributeAccessIssue]
    entrypoints_mod = plugins.entrypoints
    assert entrypoints_mod.__name__.endswith("entrypoints")  # pyright: ignore[reportAttributeAccessIssue]
    assert plugins.entrypoints is entrypoints_mod


def test_dunder_getattr_submodule_groups(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test lazy loading of the 'groups' submodule."""
    import bijux_cli.services.plugins as plugins

    if hasattr(plugins, "groups"):
        del sys.modules[plugins.__name__].groups  # pyright: ignore[reportAttributeAccessIssue]
    groups_mod = plugins.groups
    assert groups_mod.__name__.endswith("groups")  # pyright: ignore[reportAttributeAccessIssue]
    assert plugins.groups is groups_mod


def test_dunder_getattr_submodule_registry(monkeypatch: pytest.MonkeyPatch) -> None:
    """Test lazy loading of the 'registry' submodule."""
    import bijux_cli.services.plugins as plugins

    if hasattr(plugins, "registry"):
        del sys.modules[plugins.__name__].registry  # pyright: ignore[reportAttributeAccessIssue]
    registry_mod = plugins.registry
    assert registry_mod.__name__.endswith("registry")  # pyright: ignore[reportAttributeAccessIssue]
    assert plugins.registry is registry_mod
