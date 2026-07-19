# Testing And Release

`bijux-dag-cli` is small but release-critical: it is the package users install
and the final status/stream boundary for every DAG command.

## Test Layers

| Layer | What it proves |
| --- | --- |
| wrapper contract | binary name, dependencies, startup, completion availability |
| routing | non-completion commands delegate to the app |
| smoke pipeline | representative installed-binary behavior |
| app contracts | command semantics, rendering, failures, lane policy |
| runtime/core/artifact contracts | domain behavior beneath the app |

A wrapper smoke test cannot substitute for app or runtime coverage.

## Isolated Process Tests

Tests should control working directory, run root, cache root, environment
opt-ins, and fixtures. They must capture status and both streams. Test setup
must not resolve commands or state from the developer's home directory.

Use deterministic examples that exercise the installed boundary without
requiring network, cluster, container engine, or scheduler availability unless
the test is explicitly an integration lane for that dependency.

## Release Checklist

A releasable binary verifies:

- package and binary versions align with the workspace release;
- only public release dependencies are required;
- `cargo install bijux-dag-cli` produces `bijux-dag`;
- default help exposes only the stable operator surface;
- completion generation works for every declared shell;
- version, validate, and a representative local workflow preserve streams and
  status;
- command reference and app snapshots are current;
- package metadata, README, changelog, and release assets agree.

## Failure Review

When a CLI test fails, first identify ownership:

- parser/startup/completion/exit defect: fix this crate;
- route/output/config defect: fix app;
- graph semantics defect: fix core;
- execution defect: fix runtime;
- retained evidence defect: fix artifacts.

Do not patch output in the wrapper to conceal an owning-layer failure.

## Verification

```bash
cargo test --locked -p bijux-dag-cli
cargo test --locked -p bijux-dag-app --test crate_boundary_contract
```

Release validation also checks publication order, metadata, generated command
reference, and installation behavior at repository level.
