# `bijux-dag-cli` Contracts

`bijux-dag-cli` is the installable process boundary for `bijux-dag`. It owns
startup and termination behavior while delegating command meaning to
`bijux-dag-app`.

## Owned Surface

The package owns:

- the `bijux-dag` binary target;
- collection of process arguments;
- top-level command construction through the app package;
- invocation of application dispatch;
- final process exit-code mapping;
- supported shell-completion generation at the process boundary.

It does not own command-family business logic, graph validation, runtime
execution, artifact persistence, rendering policy, or maintainer commands.

## Thin Entrypoint Contract

The executable entrypoint may:

1. collect arguments and process-level context;
2. obtain the command tree from `bijux-dag-app`;
3. delegate the parsed invocation;
4. return the application's selected exit code.

It must not inspect command names to implement separate behavior, rewrite JSON
envelopes, catch failures as success, or depend directly on core, runtime, or
artifact internals.

The package depends only on `clap`, `clap_complete`, and `bijux-dag-app` for
runtime behavior. Adding another workspace dependency requires proof that the
responsibility cannot remain in its owning package.

## Compatibility Contract

The installed binary name, command tree, global options, completion behavior,
stdout/stderr discipline, and exit status are public interfaces. Their semantic
authority remains in the app package and checked-in CLI reference.

The wrapper cannot establish compatibility by itself: a green smoke test proves
startup and delegation, not every command workflow.

## Failure Contract

Argument errors use parser-owned diagnostics and nonzero status. Application
errors preserve the status selected by `bijux-dag-app`. Panics, partial output,
and successful status after failed dispatch are process-boundary defects.

## Verification

| Claim | Required evidence |
| --- | --- |
| binary startup and basic delegation | `crates/bijux-dag-cli/tests/smoke_pipeline.rs` |
| shell completion support | completion contract tests in the package and `bijux-dev` |
| command tree and exit behavior | owning `bijux-dag-app` CLI and error contracts |
| dependency thinness | `crates/bijux-dag-app/tests/crate_boundary_contract.rs` |

For wrapper changes, run:

```bash
cargo test --locked -p bijux-dag-cli
```

Any command-semantic change belongs in `bijux-dag-app` and must carry its
application and public documentation evidence there.
