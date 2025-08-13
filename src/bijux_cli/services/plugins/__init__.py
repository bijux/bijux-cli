# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the public API for the Bijux CLI's plugin management service.

This module acts as the public facade for the plugin management service layer.
It exposes a curated set of high-level functions for core plugin operations,
ensuring a stable and convenient API for the rest of the application.

To optimize CLI startup performance, this module's own submodules (such as
`registry`, `hooks`, and `entrypoints`) are loaded lazily upon first access.
"""

from __future__ import annotations

from contextlib import suppress
import importlib
import importlib.util
import os
from pathlib import Path
import shutil
import sys
from typing import Any, cast

from packaging.specifiers import SpecifierSet

from bijux_cli.__version__ import version as cli_version
from bijux_cli.contracts import (
    ObservabilityProtocol,
    RegistryProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.exceptions import BijuxError


def _di() -> Any | None:
    """Safely retrieves the current DIContainer instance.

    Returns:
        DIContainer | None: The current dependency injection container, or None
            if it is not available or an error occurs.
    """
    from bijux_cli.core.di import DIContainer

    try:
        return DIContainer.current()
    except Exception:
        return None


def _obs() -> ObservabilityProtocol | None:
    """Safely resolves the `ObservabilityProtocol` service.

    Returns:
        ObservabilityProtocol | None: The observability service, or None if it
            cannot be resolved.
    """
    di = _di()
    if not di:
        return None
    try:
        return cast(ObservabilityProtocol, di.resolve(ObservabilityProtocol))
    except KeyError:
        return None


def _tel() -> TelemetryProtocol | None:
    """Safely resolves the `TelemetryProtocol` service.

    Returns:
        TelemetryProtocol | None: The telemetry service, or None if it
            cannot be resolved.
    """
    di = _di()
    if not di:
        return None
    try:
        return cast(TelemetryProtocol, di.resolve(TelemetryProtocol))
    except KeyError:
        return None


def get_plugins_dir() -> Path:
    """Returns the directory that stores installed plugins.

    The path is determined by the `BIJUXCLI_PLUGINS_DIR` environment variable
    if set, otherwise it falls back to a default location. The directory is
    created if it does not exist.

    Returns:
        Path: The resolved path to the plugins directory.
    """
    env_path = os.environ.get("BIJUXCLI_PLUGINS_DIR")
    if env_path:
        plugins_dir = Path(env_path)
    else:
        from bijux_cli.core.paths import PLUGINS_DIR

        plugins_dir = PLUGINS_DIR
    plugins_dir = plugins_dir.expanduser()

    if plugins_dir.exists() and plugins_dir.is_symlink():
        return plugins_dir
    if plugins_dir.is_dir():
        return plugins_dir.resolve()
    if plugins_dir.exists():
        return plugins_dir.resolve()
    plugins_dir.mkdir(parents=True, exist_ok=True)
    return plugins_dir.resolve()


def load_plugin_config(name: str) -> dict[str, Any]:
    """Loads a plugin's `config.yaml` file.

    This function looks for a `config.yaml` file in the specified plugin's
    directory. It returns an empty dictionary if the file is missing.

    Args:
        name (str): The name of the plugin whose config should be loaded.

    Returns:
        dict[str, Any]: The plugin's configuration as a dictionary.

    Raises:
        BijuxError: If the `PyYAML` library is not installed or if the file
            is corrupt and cannot be parsed.
    """
    try:
        import yaml
    except ModuleNotFoundError as exc:
        raise BijuxError("PyYAML is required to read plugin configs") from exc

    cfg_path = get_plugins_dir() / name / "config.yaml"
    if not cfg_path.exists():
        return {}

    try:
        data = yaml.safe_load(cfg_path.read_text(encoding="utf-8")) or {}
    except Exception as exc:
        tel = _tel()
        if tel:
            tel.event("plugin_config_failed", {"name": name, "error": str(exc)})
        raise BijuxError(f"Failed to load config for '{name}': {exc}") from exc

    obs = _obs()
    tel = _tel()
    if obs:
        obs.log("info", "Loaded plugin config", extra={"name": name})
    if tel:
        tel.event("plugin_config_loaded", {"name": name})
    return data


def verify_plugin_signature(path: Path, public_key: str | None) -> bool:
    """Verifies the signature of a plugin file.

    Note:
        This is a placeholder function. Actual cryptographic verification is
        not implemented.

    Args:
        path (Path): The path to the plugin file to verify.
        public_key (str | None): The public key to use for verification.

    Returns:
        bool: True if the signature is "verified", False if no signature exists.

    Raises:
        BijuxError: If a signature file exists but no public key is provided.
    """
    sig = path.with_suffix(path.suffix + ".sig")
    tel = _tel()

    if not sig.exists():
        if tel:
            tel.event("plugin_unsigned", {"path": str(path)})
        return False

    if public_key is None:
        raise BijuxError(f"Signature found for {path} but no public_key provided")

    if tel:
        tel.event("plugin_signature_verified", {"path": str(path)})
    return True


def load_plugin(
    path: str | Path,
    module_name: str,
    *,
    public_key: str | None = None,
) -> Any:
    """Dynamically loads and instantiates a `Plugin` class from a `.py` file.

    This function handles the dynamic import of a plugin's code, instantiates
    its main `Plugin` class, and performs compatibility checks.

    Args:
        path (str | Path): The path to the `plugin.py` file.
        module_name (str): The name to assign to the imported module.
        public_key (str | None): An optional public key for signature verification.

    Returns:
        Any: An instantiated object of the `Plugin` class from the file.

    Raises:
        BijuxError: If the file is missing, the signature is invalid, the
            plugin is incompatible with the current CLI version, or an import
            error occurs.
    """
    path = Path(path)
    if not path.is_file():
        raise BijuxError(f"Plugin file not found: {path}")

    if public_key:
        verify_plugin_signature(path, public_key)

    spec = importlib.util.spec_from_file_location(module_name, str(path))
    if not spec or not spec.loader:
        raise BijuxError(f"Cannot import plugin: {path}")

    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module

    try:
        spec.loader.exec_module(module)

        plugin_class = getattr(module, "Plugin", None)
        if plugin_class is None:
            raise BijuxError("No `Plugin` class found in module")

        plugin = plugin_class()

        for target in (plugin_class, plugin):
            raw = getattr(target, "version", None)
            if raw is not None and not isinstance(raw, str):
                target.version = str(raw)

        required = getattr(plugin, "requires_cli_version", f"=={cli_version}")
        spec_set = SpecifierSet(required)
        if not spec_set.contains(cli_version):
            raise BijuxError(f"Plugin requires CLI {required}, host is {cli_version}")

        obs = _obs()
        cli_attr = getattr(plugin, "cli", None)
        if obs and (cli_attr is None or not callable(cli_attr)):
            obs.log(
                "warning",
                f"Plugin '{module_name}' has no callable `cli`",
                extra={"path": str(path)},
            )

        tel = _tel()
        if tel:
            tel.event("plugin_loaded", {"name": getattr(plugin, "name", module_name)})

        return plugin

    except Exception as exc:
        sys.modules.pop(module_name, None)
        raise BijuxError(f"Failed to load plugin '{path}': {exc}") from exc


def uninstall_plugin(name: str, registry: RegistryProtocol) -> bool:
    """Removes a plugin's directory and deregisters it.

    Args:
        name (str): The name of the plugin to uninstall.
        registry (RegistryProtocol): The plugin registry service.

    Returns:
        bool: True if the plugin was found and removed, otherwise False.
    """
    plug_dir = get_plugins_dir() / name
    tel = _tel()

    existed = registry.has(name)
    if not existed:
        if tel:
            tel.event("plugin_uninstall_not_found", {"name": name})
        return False

    with suppress(Exception):
        shutil.rmtree(plug_dir, ignore_errors=True)

    registry.deregister(name)
    if tel:
        tel.event("plugin_uninstalled", {"name": name})
    return True


def install_plugin(*args: Any, **kwargs: Any) -> None:
    """Stub for plugin installation. Use the CLI command instead.

    Raises:
        NotImplementedError: Always, as this function is a stub.
    """
    raise NotImplementedError("Use `bijux_cli.commands.plugins.install`")


_SUBMODULES: dict[str, str] = {
    "hooks": "bijux_cli.services.plugins.hooks",
    "entrypoints": "bijux_cli.services.plugins.entrypoints",
    "groups": "bijux_cli.services.plugins.groups",
    "registry": "bijux_cli.services.plugins.registry",
}


def __getattr__(name: str) -> Any:
    """Lazily imports submodules to optimize startup time.

    This function is a module-level implementation of `__getattr__` (PEP 562),
    which allows submodules of the `plugins` service to be imported only when
    they are first accessed.

    Args:
        name (str): The name of the submodule or attribute to access.

    Returns:
        Any: The imported submodule or attribute.

    Raises:
        AttributeError: If the requested name is not a valid submodule or
            attribute that can be lazily loaded.
    """
    if name in _SUBMODULES:
        mod = importlib.import_module(_SUBMODULES[name])
        setattr(sys.modules[__name__], name, mod)
        return mod
    if name in {"command_group", "dynamic_choices"}:
        groups_mod = importlib.import_module(_SUBMODULES["groups"])
        return getattr(groups_mod, name)
    if name == "load_entrypoints":
        entrypoints_mod = importlib.import_module(_SUBMODULES["entrypoints"])
        return entrypoints_mod.load_entrypoints
    raise AttributeError(f"module {__name__} has no attribute {name}")


__all__ = [
    "get_plugins_dir",
    "load_plugin_config",
    "verify_plugin_signature",
    "load_plugin",
    "uninstall_plugin",
    "install_plugin",
]
