# Schema Forward-Compatibility Limitations

- Future schema versions are rejected unless explicitly supported.
- Unknown required fields in stable schema payloads are treated as incompatibility.
- Forward compatibility is not implied by additive draft fields.
