# Artifact Commands

Use `artifact` commands to retrieve output evidence, lineage, and integrity signals.

## What this command family is for

`artifact` commands answer:

- what outputs were produced for this run,
- what is each artifact identity,
- what lineage links this artifact to run/node context.

## Core invocation patterns

```bash
bijux-dag artifact --help
bijux-dag artifact list --run-id RUN_20260309_220 --output json
bijux-dag artifact inspect --artifact-id ART_20260309_902 --output json
```

If your build provides trace or hash subcommands directly, prefer those for integrity automation; otherwise rely on inspect output fields.

## Metadata versus payload behavior

Commands can usually operate on metadata/lineage records. Payload-presence requirements vary by command and backend:

- metadata-only operations: listing IDs, lineage fields, stored hash values,
- payload-dependent operations: content verification and re-hash workflows.

Treat missing payload as a first-class evidence condition, not a silent omission.

## Integrity-oriented flow

```bash
bijux-dag artifact list --run-id RUN_20260309_220 --output json
bijux-dag artifact inspect --artifact-id ART_20260309_902 --output json
bijux-dag diff artifact --left ART_20260309_902 --right ART_20260309_944 --output json
```

## Next reading

- Inspect workflow integration: [Inspect Commands](../04-cli-reference/05-inspect-commands.md)
- Artifact contract semantics: [Artifact Model Specification](../06-specification/03-artifact-model.md)
