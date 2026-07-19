---
title: Error Model
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Error Model

An error result has three independent parts: a classification, an output
representation, and a process exit code. Scripts must evaluate the exit code
and the selected machine format; human-readable wording alone is not a stable
programmatic interface.

## Exit Classes

| Code | Stable class | Typical origin |
| --- | --- | --- |
| `0` | success | route completed and its result was rendered |
| `1` | error | runtime, plugin, I/O, or internal execution failure |
| `2` | usage | invalid argv, unknown route, missing argument, unsupported option, or input validation classified as usage |
| `3` | encoding | disallowed encoding or control-character input, or serialization policy failure mapped to this class |
| `130` | aborted | user interruption in an execution surface that preserves cancellation |

The process entrypoint normalizes negative internal codes to `1` and clamps
codes above `255`. Delegated processes normally preserve their native code in
`AppRunResult` before that final operating-system conversion.

## Where Failures Enter

| Boundary | Failure behavior |
| --- | --- |
| argv decoding | invalid UTF-8 is written to stderr and exits with usage |
| Clap and intent parsing | help may succeed early; malformed syntax returns usage without route execution |
| path and state resolution | malformed or inaccessible owned state fails before the handler can claim success |
| route registry | unknown, ambiguous, reserved, and conflicting namespaces are classified before execution |
| built-in handler | validation, I/O, or feature errors return through dispatch classification |
| plugin process | native stdout, stderr, timeout, and nonzero status remain a process result |
| structured rendering | an output-format failure becomes an internal app error rather than partial success output |
| stream emission | the entrypoint attempts stdout and stderr writes; host-level write failure is not converted into a second payload |

An error from one boundary must not be relabeled as success because a later
renderer or logging adapter completed.

## Structured And Text Results

Built-in route failures are rendered in the requested output format and sent
to stderr with stdout empty. Their dispatch payload includes status, numeric
code, message, and canonical command; unknown routes can also include bounded
correction fields.

The public contract types provide versioned success and error envelopes,
including stable symbolic codes, categories, details, warnings, and evidence
pointers. Not every legacy text path or delegated plugin process produces the
same envelope. Consumers that require schema stability must request a
machine-readable built-in surface documented to use that contract and validate
its schema version.

Text messages are for operators. Machine consumers should not scrape
capitalization or sentence wording to infer error categories.

## Stream Rules

- Successful built-in payloads use stdout.
- Classified built-in failures use stderr and leave stdout empty.
- Help uses stdout when the request is valid; help-topic and usage failures use
  stderr.
- Plugin and known-tool delegation preserve the child process's stream split.
- Quiet mode suppresses successful built-in output but does not turn a failure
  into success.
- A final newline is added to rendered built-in payloads when absent.

Do not combine streams in a wrapper that claims to preserve the CLI contract.
Merged logs are useful diagnostics, but they are not equivalent to the native
stdout/stderr interface.

## Classification Rules

The dispatch classifier maps known usage and encoding messages to stable
numeric classes; other handler errors map to `1`. The kernel contract carries
explicit usage, validation, plugin, and internal categories for structured
execution.

Message-based dispatch classification is therefore an implementation boundary
that requires tests whenever error wording changes. New stable failures should
prefer typed categories and symbolic codes rather than adding another
downstream string parser.

Unknown-route suggestions are bounded and deterministic. They are remediation
hints, not alternate successful routes, and the CLI does not execute the
suggested command automatically.

## Plugin Failure Boundary

Installed plugins execute as external code with the invoking user's
privileges. The host validates manifest, namespace, entrypoint, checksum, and
runtime policy, but it does not reinterpret plugin stderr as a built-in
envelope.

- timeout uses a nonzero process result;
- spawn failure is a host error;
- plugin nonzero status and streams are preserved;
- malformed structured plugin output fails validation rather than becoming a
  successful built-in payload.

These rules preserve failure ownership: host validation belongs to
`bijux-cli`; plugin program behavior belongs to the installed plugin.

## Change Discipline

Changing an exit class, stream, symbolic code, envelope field, or suggestion
shape is a compatibility change. Review the Rust contract, JSON schemas,
binary/in-process parity, Python bridge behavior, and plugin delegation in the
same change.

At minimum, verify:

- parser and global-flag normalization failures;
- unknown-route and suggestion behavior;
- text and machine-readable stream placement;
- plugin timeout, spawn, and nonzero process results;
- binary and in-process exit parity;
- quiet-mode invariance;
- encoding and interruption classes.

## Implementation Anchors

- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/interface/cli/dispatch/policy.rs`
- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/src/contracts/envelope.rs`
- `crates/bijux-cli/src/contracts/execution.rs`
- `crates/bijux-cli/src/features/plugins/runtime.rs`
- `crates/bijux-cli/src/bootstrap/run.rs`
- `crates/bijux-cli/tests/integration/cli/root/`

## Related Contracts

- [Data Contracts](../interfaces/data-contracts.md)
- [Execution Model](execution-model.md)
- [Failure Recovery](../operations/failure-recovery.md)
- [Invariants](../quality/invariants.md)
