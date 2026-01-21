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

from bijux_cli.core.contracts import (
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
    import bijux_cli.services.logging.emitter
    import bijux_cli.services.logging.observability
    import bijux_cli.services.diagnostics.process
    import bijux_cli.services.diagnostics.retry
    import bijux_cli.services.logging.serializer
    import bijux_cli.services.logging.telemetry
    import bijux_cli.services.diagnostics.audit
    import bijux_cli.services.config
    import bijux_cli.services.diagnostics.docs
    import bijux_cli.services.diagnostics.doctor
    import bijux_cli.services.history
    import bijux_cli.services.diagnostics.memory
    import bijux_cli.plugins.registry

    obs_service = bijux_cli.services.logging.observability.Observability(debug=debug)

    di.register(bijux_cli.services.logging.observability.Observability, lambda: obs_service)
    di.register(
        ObservabilityProtocol,
        lambda: di.resolve(bijux_cli.services.logging.observability.Observability),
    )

    di.register(
        bijux_cli.services.logging.telemetry.LoggingTelemetry,
        lambda: bijux_cli.services.logging.telemetry.LoggingTelemetry(
            observability=di.resolve(bijux_cli.services.logging.observability.Observability)
        ),
    )
    di.register(
        TelemetryProtocol,
        lambda: di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
    )

    di.register(
        bijux_cli.services.logging.emitter.Emitter,
        lambda: bijux_cli.services.logging.emitter.Emitter(
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
            output_format=output_format,
            debug=debug,
            quiet=quiet,
        ),
    )
    di.register(EmitterProtocol, lambda: di.resolve(bijux_cli.services.logging.emitter.Emitter))

    di.register(
        bijux_cli.services.logging.serializer.OrjsonSerializer,
        lambda: bijux_cli.services.logging.serializer.OrjsonSerializer(
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        bijux_cli.services.logging.serializer.PyYAMLSerializer,
        lambda: bijux_cli.services.logging.serializer.PyYAMLSerializer(
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        SerializerProtocol,
        lambda: (
            di.resolve(bijux_cli.services.logging.serializer.OrjsonSerializer)
            if output_format is OutputFormat.JSON
            else di.resolve(bijux_cli.services.logging.serializer.PyYAMLSerializer)
        ),
    )

    di.register(
        bijux_cli.services.diagnostics.process.ProcessPool,
        lambda: bijux_cli.services.diagnostics.process.ProcessPool(
            observability=di.resolve(bijux_cli.services.logging.observability.Observability),
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
        ),
    )
    di.register(
        ProcessPoolProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.process.ProcessPool)
    )

    di.register(
        bijux_cli.services.diagnostics.retry.TimeoutRetryPolicy,
        lambda: bijux_cli.services.diagnostics.retry.TimeoutRetryPolicy(
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        bijux_cli.services.diagnostics.retry.ExponentialBackoffRetryPolicy,
        lambda: bijux_cli.services.diagnostics.retry.ExponentialBackoffRetryPolicy(
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        RetryPolicyProtocol,
        lambda: di.resolve(bijux_cli.services.diagnostics.retry.TimeoutRetryPolicy),
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
        bijux_cli.plugins.registry.Registry,
        lambda: bijux_cli.plugins.registry.Registry(
            di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry)
        ),
    )
    di.register(
        RegistryProtocol,
        lambda: di.resolve(bijux_cli.plugins.registry.Registry),
    )

    di.register(
        bijux_cli.services.diagnostics.audit.DryRunAudit,
        lambda: bijux_cli.services.diagnostics.audit.DryRunAudit(
            di.resolve(bijux_cli.services.logging.observability.Observability),
            di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
        ),
    )
    di.register(
        bijux_cli.services.diagnostics.audit.RealAudit,
        lambda: bijux_cli.services.diagnostics.audit.RealAudit(
            di.resolve(bijux_cli.services.logging.observability.Observability),
            di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
        ),
    )
    di.register(
        AuditProtocol,
        lambda: bijux_cli.services.diagnostics.audit.get_audit_service(
            observability=di.resolve(bijux_cli.services.logging.observability.Observability),
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
            dry_run=False,
        ),
    )

    di.register(
        bijux_cli.services.diagnostics.docs.Docs,
        lambda: bijux_cli.services.diagnostics.docs.Docs(
            observability=di.resolve(bijux_cli.services.logging.observability.Observability),
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
        ),
    )
    di.register(DocsProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.docs.Docs))

    di.register(
        bijux_cli.services.diagnostics.doctor.Doctor,
        lambda: bijux_cli.services.diagnostics.doctor.Doctor(),
    )
    di.register(DoctorProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.doctor.Doctor))

    di.register(
        bijux_cli.services.history.History,
        lambda: bijux_cli.services.history.History(
            telemetry=di.resolve(bijux_cli.services.logging.telemetry.LoggingTelemetry),
            observability=di.resolve(bijux_cli.services.logging.observability.Observability),
        ),
    )
    di.register(HistoryProtocol, lambda: di.resolve(bijux_cli.services.history.History))

    di.register(
        bijux_cli.services.diagnostics.memory.Memory,
        lambda: bijux_cli.services.diagnostics.memory.Memory(),
    )
    di.register(MemoryProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.memory.Memory))


__all__ = ["register_default_services"]
