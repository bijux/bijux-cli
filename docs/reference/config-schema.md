# Config schema

## Purpose
This document guarantees the configuration key space and contracts.

## Scope
It covers config keys and contract ownership only.

## Core Concepts
- Keys are stored in a dotenv-style file.
- Keys are uppercased with a `BIJUXCLI_` prefix.

## Invariants
- Keys use alphanumeric or underscore only.
- Duplicate keys are invalid.

## Execution
Common keys:

| Key | Type | Notes |
| --- | --- | --- |
| format | string | json or yaml |
| log_level | string | trace, debug, info |
| color | string | auto, always, never |

## Failure Modes
- Invalid keys exit with code 2.
- Non-ASCII values exit with code 3.

## Design Rationale
- Alternatives: arbitrary key formats.
- Rejected because they break validation.

## Non-Goals
- Dynamic schema discovery.

## Contracts
Core contracts:

- Execution: `ContextProtocol`
- Plugin interface: `RegistryProtocol`
- Retry policy: `RetryPolicyProtocol`
- Serializer: `SerializerProtocol`
- Logging facade: `ObservabilityProtocol`
- Emitter: `EmitterProtocol`
- Filesystem: `FileSystemProtocol`
- Process runner: `ProcessPoolProtocol`
- Telemetry: `TelemetryProtocol`
- Terminal: `TerminalProtocol`

Service contracts:

- Config: `ConfigProtocol`
- Diagnostics: `AuditProtocol`, `DocsProtocol`, `DoctorProtocol`, `MemoryProtocol`
- History: `HistoryProtocol`
- Logging: `LoggingConfig`
- Plugins: `PluginConfig`

Infra adapters:

- `EmitterProtocol` -> `ConsoleEmitter`
- `SerializerProtocol` -> `OrjsonSerializer`, `PyYAMLSerializer`
- `RetryPolicyProtocol` -> `TimeoutRetryPolicy`, `ExponentialBackoffRetryPolicy`
- `ProcessPoolProtocol` -> `ProcessPool`
- `TerminalProtocol` -> `Terminal`
- `TelemetryProtocol` -> `NoopTelemetry`, `LoggingTelemetry`
