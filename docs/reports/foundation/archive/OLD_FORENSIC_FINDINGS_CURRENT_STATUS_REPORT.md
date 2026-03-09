# Old Forensic Findings: Current Status

This report re-audits prior forensic findings against the current code and test suite.

## Container Image Validation Rules

- Empty or whitespace-only image strings are rejected.
- Non-empty image strings are treated as literal image identifiers, including values that begin with `-` or contain option-like substrings.
- Runtime argument construction passes the image as a positional argument after engine flags.

## Fixed

- Container contract accepts literal image values that start with `-` and option-like segments (`container_execution_contracts`).
- Runtime adapters use centralized environment shaping in shell, container, and external execution paths (`shape_environment` routing).
- Output validation rejects symlinked intermediate components and skips symlink loops while scanning undeclared outputs.
- External adapter transport rejects oversized serialized `--node-spec` payloads.
- Cache pack extraction rejects oversized archives and hostile entry types (non-file, non-directory).

## Stale

- Previous concern that evidence-foundation output is opaque is stale; a stepwise verification summary is now generated at `artifacts/reports/evidence-foundation-verification-summary.md`.

## Needs More Work

- Timeout defaults from global runtime config are still not threaded into adapter-local timeout execution when a node-level timeout is absent.
- External adapter node-spec minimization/redaction policy is still broad; current guard enforces payload size but does not reduce data shape.

## Still Open

- None classified as release-blocking after this pass; remaining items are hardening follow-ups tracked under "Needs More Work".
