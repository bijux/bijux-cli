# Config schema

Keys are stored in the config file and resolved by precedence.
Keys are uppercased with a `BIJUXCLI_` prefix and use alphanumeric or underscore.

Common keys:

| Key | Type | Notes |
| --- | --- | --- |
| format | string | json or yaml |
| log_level | string | trace, debug, info |
| color | string | auto, always, never |

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

- `EmitterProtocol` → `ConsoleEmitter`
- `SerializerProtocol` → `OrjsonSerializer`, `PyYAMLSerializer`
- `RetryPolicyProtocol` → `TimeoutRetryPolicy`, `ExponentialBackoffRetryPolicy`
- `ProcessPoolProtocol` → `ProcessPool`
- `TerminalProtocol` → `Terminal`
- `TelemetryProtocol` → `NoopTelemetry`, `LoggingTelemetry`
