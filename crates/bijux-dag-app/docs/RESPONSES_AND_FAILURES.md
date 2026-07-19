# Responses And Failures

Every application workflow has one semantic outcome. Human and machine
renderers may present it differently but must agree on success, failure class,
identity, diagnostics, and retained evidence.

## Typed Responses

Command families define response models before rendering, including validation,
graph, run, status, doctor, export, import, replay, and diff responses. Shared
report contracts carry stable command data for consumers.

Renderers receive typed data. They must not reopen run directories, recompute
domain facts, or change status based on formatting.

## JSON Contract

Commands promising JSON emit one parseable envelope on stdout for success and
failure. Incidental diagnostics belong on stderr. The envelope preserves:

- command identity;
- success flag and exit classification;
- typed result data;
- structured diagnostics;
- relevant run, node, artifact, or path identity.

JSON mode cannot emit a human preamble or partial document.

## Human Contract

Human output prioritizes the decision and next action. It may add labels,
tables, summaries, bounded log tails, and recovery guidance. It cannot omit
the causal failure, claim unsupported guarantees, or hide an evidence refusal.

Quiet mode suppresses allowed presentation but does not alter outcome.

## Failure Classes

The app distinguishes:

- malformed command or source input;
- graph validation or compatibility refusal;
- configuration or policy refusal;
- unsupported lane, backend, or capability;
- runtime launch, timeout, cancellation, node, or persistence failure;
- missing, corrupt, or incompatible evidence;
- unsafe path or destination;
- rendering or internal defect.

Operator-controlled input must not panic. Domain classifications map to stable
exit behavior without message-string guessing where typed errors exist.

## Causality And Recovery

Failure summaries separate root causes from propagated skips and later
consequences. Retry, repair, resume, cache, and replay reporting preserves the
original failed attempt.

Guidance states the required action and evidence location. It does not promise
that retry will succeed or that repair restores semantic equivalence.

## Reference Generation

`write_checked_in_cli_reference_docs` derives command reference from the
current command model. Generated reference drift is a test failure. A command
change updates model, generated reference, output fixtures, and compatibility
notes together.

## Verification

```bash
cargo test --locked -p bijux-dag-app --test output_contract
cargo test --locked -p bijux-dag-app --test error_output_contract
cargo test --locked -p bijux-dag-app --test operator_schema_lockstep_contracts
cargo test --locked -p bijux-dag-app --test operator_input_no_panic_contracts
```

Human and error snapshots require semantic review; snapshot refresh alone is
not evidence that changed wording remains truthful.
