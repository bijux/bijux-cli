# SPDX-License-Identifier: Apache-2.0
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

import os
from typing import TYPE_CHECKING, cast

from bijux_cli.core.contracts import ExecutionContext
from bijux_cli.core.enums import OutputFormat
from bijux_cli.infra.contracts import Emitter, ProcessRunner, RetryPolicy, Serializer
from bijux_cli.services.config.contracts import ConfigProtocol
from bijux_cli.services.contracts import ObservabilityProtocol, TelemetryProtocol
from bijux_cli.services.diagnostics.contracts import (
    AuditProtocol,
    DiagnosticsConfig,
    DocsProtocol,
    DoctorProtocol,
    MemoryProtocol,
)
from bijux_cli.services.history.contracts import HistoryProtocol
from bijux_cli.services.logging.contracts import LoggingConfig

if TYPE_CHECKING:
    from bijux_cli.core.di import DIContainer
    from bijux_cli.core.enums import OutputFormat


def register_default_services(
    di: DIContainer,
    *,
    logging_config: LoggingConfig,
    output_format: OutputFormat,
    diagnostics_config: DiagnosticsConfig | None = None,
) -> None:
    """Registers all default service implementations with the DI container.

    This function populates the container with lazy-loading factories for each
    core service the application requires, from configuration and logging to
    plugin management and command history.

    Args:
        di (DIContainer): The dependency injection container instance.
        logging_config (LoggingConfig): Logging configuration for services.
        output_format (OutputFormat): The default output format for services
            like the emitter and serializer.

    Returns:
        None:
    """
    import bijux_cli.core.context
    import bijux_cli.infra.emitter
    import bijux_cli.infra.process
    import bijux_cli.infra.retry
    import bijux_cli.infra.serializer
    import bijux_cli.infra.telemetry
    import bijux_cli.services.config
    import bijux_cli.services.diagnostics.audit
    import bijux_cli.services.diagnostics.docs
    import bijux_cli.services.diagnostics.doctor
    import bijux_cli.services.diagnostics.memory
    import bijux_cli.services.diagnostics.telemetry
    import bijux_cli.services.history
    import bijux_cli.services.logging.observability

    if diagnostics_config is None:
        diagnostics_config = DiagnosticsConfig(enabled=True, telemetry_enabled=True)

    di.register(DiagnosticsConfig, lambda: diagnostics_config)

    noop_telemetry = bijux_cli.infra.telemetry.NoopTelemetry()
    obs_service = bijux_cli.services.logging.observability.Observability(
        log_level=logging_config.log_level,
        telemetry=noop_telemetry,
    )

    di.register(
        bijux_cli.services.logging.observability.Observability, lambda: obs_service
    )
    di.register(
        ObservabilityProtocol,
        lambda: di.resolve(bijux_cli.services.logging.observability.Observability),
    )

    di.register(
        bijux_cli.infra.telemetry.LoggingTelemetry,
        lambda: bijux_cli.infra.telemetry.LoggingTelemetry(
            observability=di.resolve(
                bijux_cli.services.logging.observability.Observability
            )
        ),
    )
    di.register(bijux_cli.infra.telemetry.NoopTelemetry, lambda: noop_telemetry)
    di.register(
        TelemetryProtocol,
        lambda: bijux_cli.services.diagnostics.telemetry.resolve_telemetry(di),
    )

    di.register(
        bijux_cli.infra.emitter.ConsoleEmitter,
        lambda: bijux_cli.infra.emitter.ConsoleEmitter(
            telemetry=di.resolve(TelemetryProtocol),
            output_format=output_format,
        ),
    )
    di.register(Emitter, lambda: di.resolve(bijux_cli.infra.emitter.ConsoleEmitter))

    di.register(
        bijux_cli.infra.serializer.OrjsonSerializer,
        lambda: bijux_cli.infra.serializer.OrjsonSerializer(
            telemetry=di.resolve(TelemetryProtocol)
        ),
    )
    di.register(
        bijux_cli.infra.serializer.PyYAMLSerializer,
        lambda: bijux_cli.infra.serializer.PyYAMLSerializer(
            telemetry=di.resolve(TelemetryProtocol)
        ),
    )
    di.register(
        Serializer,
        lambda: (
            di.resolve(bijux_cli.infra.serializer.OrjsonSerializer)
            if output_format is OutputFormat.JSON
            else di.resolve(bijux_cli.infra.serializer.PyYAMLSerializer)
        ),
    )

    di.register(
        bijux_cli.infra.process.ProcessPool,
        lambda: bijux_cli.infra.process.ProcessPool(
            observability=di.resolve(
                bijux_cli.services.logging.observability.Observability
            ),
            telemetry=di.resolve(TelemetryProtocol),
            max_workers=int(os.getenv("BIJUXCLI_MAX_WORKERS", "4")),
            allowed_commands=os.getenv(
                "BIJUXCLI_ALLOWED_COMMANDS", "echo,ls,cat,grep"
            ).split(","),
        ),
    )
    di.register(ProcessRunner, lambda: di.resolve(bijux_cli.infra.process.ProcessPool))

    di.register(
        bijux_cli.infra.retry.TimeoutRetryPolicy,
        lambda: bijux_cli.infra.retry.TimeoutRetryPolicy(
            telemetry=di.resolve(TelemetryProtocol)
        ),
    )
    di.register(
        bijux_cli.infra.retry.ExponentialBackoffRetryPolicy,
        lambda: bijux_cli.infra.retry.ExponentialBackoffRetryPolicy(
            telemetry=di.resolve(TelemetryProtocol)
        ),
    )
    di.register(
        RetryPolicy,
        lambda: cast(
            RetryPolicy,
            di.resolve(bijux_cli.infra.retry.TimeoutRetryPolicy),
        ),
    )

    di.register(LoggingConfig, lambda: logging_config)

    di.register(
        bijux_cli.core.context.Context,
        lambda: bijux_cli.core.context.Context(di),
    )
    di.register(ExecutionContext, lambda: di.resolve(bijux_cli.core.context.Context))

    di.register(
        bijux_cli.services.config.Config,
        lambda: bijux_cli.services.config.Config(di),
    )
    di.register(ConfigProtocol, lambda: di.resolve(bijux_cli.services.config.Config))

    di.register(
        bijux_cli.services.diagnostics.audit.DryRunAudit,
        lambda: bijux_cli.services.diagnostics.audit.DryRunAudit(
            di.resolve(bijux_cli.services.logging.observability.Observability),
            di.resolve(TelemetryProtocol),
        ),
    )
    di.register(
        bijux_cli.services.diagnostics.audit.RealAudit,
        lambda: bijux_cli.services.diagnostics.audit.RealAudit(
            di.resolve(bijux_cli.services.logging.observability.Observability),
            di.resolve(TelemetryProtocol),
            allowed_commands=os.getenv(
                "BIJUXCLI_ALLOWED_COMMANDS", "echo,ls,cat,grep"
            ).split(","),
        ),
    )
    di.register(
        AuditProtocol,
        lambda: bijux_cli.services.diagnostics.audit.get_audit_service(
            observability=di.resolve(
                bijux_cli.services.logging.observability.Observability
            ),
            telemetry=di.resolve(TelemetryProtocol),
            dry_run=False,
        ),
    )

    di.register(
        bijux_cli.services.diagnostics.docs.Docs,
        lambda: bijux_cli.services.diagnostics.docs.Docs(
            observability=di.resolve(
                bijux_cli.services.logging.observability.Observability
            ),
            serializer=di.resolve(Serializer),
            telemetry=di.resolve(TelemetryProtocol),
        ),
    )
    di.register(
        DocsProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.docs.Docs)
    )

    di.register(
        bijux_cli.services.diagnostics.doctor.Doctor,
        lambda: bijux_cli.services.diagnostics.doctor.Doctor(),
    )
    di.register(
        DoctorProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.doctor.Doctor)
    )

    di.register(
        bijux_cli.services.history.History,
        lambda: bijux_cli.services.history.History(
            telemetry=di.resolve(TelemetryProtocol),
            observability=di.resolve(
                bijux_cli.services.logging.observability.Observability
            ),
        ),
    )
    di.register(HistoryProtocol, lambda: di.resolve(bijux_cli.services.history.History))

    di.register(
        bijux_cli.services.diagnostics.memory.Memory,
        lambda: bijux_cli.services.diagnostics.memory.Memory(),
    )
    di.register(
        MemoryProtocol, lambda: di.resolve(bijux_cli.services.diagnostics.memory.Memory)
    )


__all__ = ["register_default_services"]
