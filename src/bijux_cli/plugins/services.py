# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin service registration."""

from __future__ import annotations

from typing import TYPE_CHECKING

from bijux_cli.plugins.contracts import PluginConfig, RegistryProtocol
from bijux_cli.services.contracts import TelemetryProtocol

if TYPE_CHECKING:
    from bijux_cli.core.di import DIContainer


def register_plugin_services(
    di: DIContainer, *, plugin_config: PluginConfig | None = None
) -> None:
    """Register plugin configuration and registry services."""
    import bijux_cli.plugins.registry

    if plugin_config is None:
        plugin_config = PluginConfig(enabled=True, allow_entrypoints=True)

    di.register(PluginConfig, lambda: plugin_config)
    di.register(
        bijux_cli.plugins.registry.Registry,
        lambda: bijux_cli.plugins.registry.Registry(di.resolve(TelemetryProtocol)),
    )
    di.register(
        RegistryProtocol,
        lambda: di.resolve(bijux_cli.plugins.registry.Registry),
    )


__all__ = ["register_plugin_services"]
