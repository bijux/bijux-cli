# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Registers the default services for the Bijux CLI application.

This module serves as the primary composition root for the application's
service layer. It provides a single function, `register_default_services`,
which is responsible for binding all core service protocols to their
concrete implementations within the Dependency Injection (DI) container.

This centralized registration is a key part of the application's Inversion of
Control (IoC) architecture, allowing components to be easily swapped or mocked
for testing.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from bijux_cli.contracts import (
    AuditProtocol,
    ConfigProtocol,
    ContextProtocol,
    DocsProtocol,
    DoctorProtocol,
    EmitterProtocol,
    HistoryProtocol,
    MemoryProtocol,
    ObservabilityProtocol,
    ProcessPoolProtocol,
    RegistryProtocol,
    RetryPolicyProtocol,
    SerializerProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.enums import OutputFormat

if TYPE_CHECKING:
    from bijux_cli.core.di import DIContainer
    from bijux_cli.core.enums import OutputFormat


def register_default_services(
    di: DIContainer, debug: bool, output_format: OutputFormat, quiet: bool
) -> None:
    """Registers all default service implementations with the DI container.

    This function populates the container with lazy-loading factories for each
    core service the application requires, from configuration and logging to
    plugin management and command history.

    Args:
        di (DIContainer): The dependency injection container instance.
        debug (bool): If True, services will be configured for debug mode.
        output_format (OutputFormat): The default output format for services
            like the emitter and serializer.
        quiet (bool): If True, services will be configured to suppress output.

    Returns:
        None:
    """
    import bijux_cli.core.context
    import bijux_cli.infra.emitter
    import bijux_cli.infra.observability
    import bijux_cli.infra.process
    import bijux_cli.infra.retry
    import bijux_cli.infra.serializer
    import bijux_cli.infra.telemetry
    import bijux_cli.services.audit
    import bijux_cli.services.config
    import bijux_cli.services.docs
    import bijux_cli.services.doctor
    import bijux_cli.services.history
    import bijux_cli.services.memory
    import bijux_cli.services.plugins.registry

    obs_service = bijux_cli.infra.observability.Observability(debug=debug)

    di.register(bijux_cli.infra.observability.Observability, lambda: obs_service)
    di.register(
        ObservabilityProtocol,
        lambda: di.resolve(bijux_cli.infra.observability.Observability),
    )

    di.register(
        bijux_cli.infra.telemetry.LoggingTelemetry,
        lambda: bijux_cli.infra.telemetry.LoggingTelemetry(
            observability=di.resolve(bijux_cli.infra.observability.Observability)
        ),
    )
    di.register(
        TelemetryProtocol,
        lambda: di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
    )

    di.register(
        bijux_cli.infra.emitter.Emitter,
        lambda: bijux_cli.infra.emitter.Emitter(
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
            format=output_format,
            debug=debug,
            quiet=quiet,
        ),
    )
    di.register(EmitterProtocol, lambda: di.resolve(bijux_cli.infra.emitter.Emitter))

    di.register(
        bijux_cli.infra.serializer.OrjsonSerializer,
        lambda: bijux_cli.infra.serializer.OrjsonSerializer(
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        bijux_cli.infra.serializer.PyYAMLSerializer,
        lambda: bijux_cli.infra.serializer.PyYAMLSerializer(
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        SerializerProtocol,
        lambda: (
            di.resolve(bijux_cli.infra.serializer.OrjsonSerializer)
            if output_format is OutputFormat.JSON
            else di.resolve(bijux_cli.infra.serializer.PyYAMLSerializer)
        ),
    )

    di.register(
        bijux_cli.infra.process.ProcessPool,
        lambda: bijux_cli.infra.process.ProcessPool(
            observability=di.resolve(bijux_cli.infra.observability.Observability),
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
        ),
    )
    di.register(
        ProcessPoolProtocol, lambda: di.resolve(bijux_cli.infra.process.ProcessPool)
    )

    di.register(
        bijux_cli.infra.retry.TimeoutRetryPolicy,
        lambda: bijux_cli.infra.retry.TimeoutRetryPolicy(
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        bijux_cli.infra.retry.ExponentialBackoffRetryPolicy,
        lambda: bijux_cli.infra.retry.ExponentialBackoffRetryPolicy(
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        RetryPolicyProtocol,
        lambda: di.resolve(bijux_cli.infra.retry.TimeoutRetryPolicy),
    )

    di.register(
        bijux_cli.core.context.Context,
        lambda: bijux_cli.core.context.Context(di),
    )
    di.register(ContextProtocol, lambda: di.resolve(bijux_cli.core.context.Context))

    di.register(
        bijux_cli.services.config.Config,
        lambda: bijux_cli.services.config.Config(di),
    )
    di.register(ConfigProtocol, lambda: di.resolve(bijux_cli.services.config.Config))

    di.register(
        bijux_cli.services.plugins.registry.Registry,
        lambda: bijux_cli.services.plugins.registry.Registry(
            di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        RegistryProtocol,
        lambda: di.resolve(bijux_cli.services.plugins.registry.Registry),
    )

    di.register(
        bijux_cli.services.audit.DryRunAudit,
        lambda: bijux_cli.services.audit.DryRunAudit(
            di.resolve(bijux_cli.infra.observability.Observability),
            di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
        ),
    )
    di.register(
        bijux_cli.services.audit.RealAudit,
        lambda: bijux_cli.services.audit.RealAudit(
            di.resolve(bijux_cli.infra.observability.Observability),
            di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
        ),
    )
    di.register(
        AuditProtocol,
        lambda: bijux_cli.services.audit.get_audit_service(
            observability=di.resolve(bijux_cli.infra.observability.Observability),
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
            dry_run=False,
        ),
    )

    di.register(
        bijux_cli.services.docs.Docs,
        lambda: bijux_cli.services.docs.Docs(
            observability=di.resolve(bijux_cli.infra.observability.Observability),
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
        ),
    )
    di.register(DocsProtocol, lambda: di.resolve(bijux_cli.services.docs.Docs))

    di.register(
        bijux_cli.services.doctor.Doctor,
        lambda: bijux_cli.services.doctor.Doctor(),
    )
    di.register(DoctorProtocol, lambda: di.resolve(bijux_cli.services.doctor.Doctor))

    di.register(
        bijux_cli.services.history.History,
        lambda: bijux_cli.services.history.History(
            telemetry=di.resolve(bijux_cli.infra.telemetry.LoggingTelemetry),
            observability=di.resolve(bijux_cli.infra.observability.Observability),
        ),
    )
    di.register(HistoryProtocol, lambda: di.resolve(bijux_cli.services.history.History))

    di.register(
        bijux_cli.services.memory.Memory,
        lambda: bijux_cli.services.memory.Memory(),
    )
    di.register(MemoryProtocol, lambda: di.resolve(bijux_cli.services.memory.Memory))


__all__ = ["register_default_services"]
