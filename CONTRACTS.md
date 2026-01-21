# Contracts Index

This document maps contract ownership and the concrete adapters that implement
them. It is intentionally short and stable to prevent drift.

## Core contracts (cross-cutting, infra-facing)

- Execution: `ContextProtocol` (`src/bijux_cli/core/contracts_pkg/context.py`)
- Plugin interface: `RegistryProtocol` (`src/bijux_cli/core/contracts_pkg/registry.py`)
- Retry policy: `RetryPolicyProtocol` (`src/bijux_cli/core/contracts_pkg/retry.py`)
- Serializer: `SerializerProtocol` (`src/bijux_cli/core/contracts_pkg/serializer.py`)
- Logging facade: `ObservabilityProtocol` (`src/bijux_cli/core/contracts_pkg/observability.py`)
- Emitter: `EmitterProtocol` (`src/bijux_cli/core/contracts_pkg/emitter.py`)
- Filesystem: `FileSystemProtocol` (`src/bijux_cli/core/contracts_pkg/fs.py`)
- Process runner: `ProcessPoolProtocol` (`src/bijux_cli/core/contracts_pkg/process.py`)
- Telemetry: `TelemetryProtocol` (`src/bijux_cli/core/contracts_pkg/telemetry.py`)
- Terminal: `TerminalProtocol` (`src/bijux_cli/core/contracts_pkg/terminal.py`)

## Service contracts (per-service)

- Config: `ConfigProtocol` (`src/bijux_cli/services/config/contracts.py`)
- Diagnostics: `AuditProtocol`, `DocsProtocol`, `DoctorProtocol`, `MemoryProtocol`,
  `DiagnosticsConfig` (`src/bijux_cli/services/diagnostics/contracts.py`)
- History: `HistoryProtocol` (`src/bijux_cli/services/history/contracts.py`)
- Logging: `LoggingConfig` (`src/bijux_cli/services/logging/contracts.py`)
- Plugins: `PluginConfig` (`src/bijux_cli/services/plugins/contracts.py`)

## Infra adapters (concrete implementations only)

- `EmitterProtocol` → `ConsoleEmitter` (`src/bijux_cli/infra/emitter.py`)
- `SerializerProtocol` → `OrjsonSerializer`, `PyYAMLSerializer`
  (`src/bijux_cli/infra/serializer.py`)
- `RetryPolicyProtocol` → `TimeoutRetryPolicy`, `ExponentialBackoffRetryPolicy`
  (`src/bijux_cli/infra/retry.py`)
- `ProcessPoolProtocol` → `ProcessPool` (`src/bijux_cli/infra/process.py`)
- `TerminalProtocol` → `Terminal` (`src/bijux_cli/infra/terminal.py`)
- `TelemetryProtocol` → `NoopTelemetry`, `LoggingTelemetry`
  (`src/bijux_cli/infra/telemetry.py`)
