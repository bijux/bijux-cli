# Plugin model

Plugins extend the CLI with new commands and behaviors. Plugins are validated
before activation.

Lifecycle states:

- discovered
- installed
- active
- inactive
- removed

Rules:

- Metadata is validated before loading
- Invalid metadata fails fast
- Activation and removal are explicit

Compatibility:

- Plugins declare CLI compatibility in metadata
- Incompatible plugins are rejected
