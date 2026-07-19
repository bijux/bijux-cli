# Routing And Execution

Routing turns operator input into one canonical execution decision. The
runtime keeps normalization, route ownership, execution policy, and handler
effects separate so that help, inspection, completion, and execution describe
the same command graph.

## Canonical Route Authority

`routing/catalog.rs` owns root aliases, recognized subcommands, normalization,
and REPL reference commands. `routing/model.rs` assembles built-in paths and
alias rewrites. `routing/registry.rs` resolves registered routes and their
source metadata.

Every consumer must use these authorities instead of maintaining an adjacent
list. A command that appears in help but cannot be resolved, or executes but is
absent from inspection, is a contract defect.

## Resolution Order

An invocation is resolved in this order:

1. normalize the executable name, root aliases, and command path;
2. parse global flags without changing command identity;
3. resolve an exact built-in route;
4. evaluate mounted-product and plugin descriptors under namespace policy;
5. reject unknown, ambiguous, incompatible, or reserved routes;
6. construct the kernel execution intent and policy.

Conflict resolution must be deterministic. Filesystem enumeration order,
registry insertion order, and environment ordering cannot determine which
handler wins.

## Execution Policy

The kernel combines defaults, configuration, environment, and explicit flags
into one `ExecutionPolicy`. Explicit invocation values have the strongest
precedence. Policy covers output format, pretty and color modes, logging,
quiet behavior, tracing, timeout, and related execution controls.

`ExecutionIntent` records canonical command identity. `ExecutionContext`
combines the intent with resolved policy and cancellation state. Handlers
receive that context rather than reading command-line globals independently.

## Handler Lifecycle

The kernel supports synchronous and asynchronous handlers behind one
normalized outcome model. The pipeline:

- runs lifecycle hooks in stable order;
- short-circuits pre-execution cancellation;
- catches handler panics and reports internal failure;
- applies timeout and post-dispatch cancellation rules;
- normalizes success or error payloads;
- maps the normalized category to an owned exit code;
- emits bounded diagnostics without changing the result shape.

Quiet mode may suppress streams but cannot turn failure into success. Trace
mode may add diagnostics but cannot mutate the command payload.

## Plugin And Mounted Routes

External routes are accepted only after descriptor validation, compatibility
evaluation, reserved-namespace checks, and trust-policy enforcement. Process
launch is an effect after route selection, never a discovery mechanism.

Failures remain distinguishable:

- unknown or ambiguous route;
- malformed or incompatible descriptor;
- unavailable executable or interpreter;
- external non-zero status;
- invalid external output;
- host runtime failure.

## Change Checklist

- Update the canonical catalog or registry, not a secondary list.
- Add parser and registry tests for aliases and conflict behavior.
- Verify help, inspect, completion, and execution parity.
- Preserve deterministic route provenance.
- Test sync and async handler equivalence where both paths apply.
- Test cancellation, timeout, panic, and exit-code behavior for new handlers.

## Verification

```bash
cargo test --locked -p bijux-cli --test routing
cargo test --locked -p bijux-cli kernel::tests
```

Command-level behavior belongs in focused suites under
`tests/integration.rs`, especially when a route crosses plugin, mount, REPL,
or configuration boundaries.
