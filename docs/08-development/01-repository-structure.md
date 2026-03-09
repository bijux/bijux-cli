# Repository Structure

## Purpose
Define the repository layout and ownership boundaries for maintainers.

## Context
A clear repository map reduces coupling, prevents misplaced changes, and supports predictable maintenance.

## Explanation
Top-level structure intent:
- `crates/`: Rust crates grouped by domain responsibility.
- `docs/`: reader-facing documentation and contracts.
- `examples/` or fixtures directories: deterministic samples and validation inputs.
- CI and tooling manifests: build, test, and release automation entrypoints.

Developer environment setup baseline:
- install pinned Rust toolchain used by CI.
- enable local `cargo` lockfile-aware workflows (`--locked`) for reproducibility checks.
- verify required tooling before contribution (`fmt`, `clippy`, test toolchain).
- keep local branch rebased to reduce cross-domain merge conflicts.

Crate boundary expectations:
- command-surface crates own CLI and operator interfaces.
- runtime crates own scheduling, execution, and run lifecycle behavior.
- control-plane crates own repository and governance automation behavior.
- shared libraries own reusable contracts and pure utilities.

Placement rules:
- put feature code in crate with matching domain ownership.
- avoid cross-cutting changes in unrelated crates unless contract updates require it.
- keep docs close to stable domain terms, not temporary refactor language.

Change review rules:
- boundary-crossing changes must document why boundary updates are necessary.
- rename/move operations must preserve clear ownership history in commit messages.

## Examples
```text
Example mapping:
crates/bijux-dag-runtime      -> execution and scheduler behavior
crates/bijux-dag-cli          -> user command surface
crates/bijux-dev-dag          -> control-plane workflows
docs/06-specification         -> canonical behavior contracts
```

```text
Good ownership check:
"Does this change belong to runtime semantics or CLI presentation?"
```

## Guarantees
- Repository domains and expected ownership boundaries are explicit.
- Contributors can place changes with lower risk of architecture drift.
- Boundary crossing requires deliberate documentation and review clarity.
- Includes minimum developer setup guidance tied to repository structure usage.

## Limitations
- This guide does not list every file path in the repository.
- Exact internal module layout can evolve without changing domain ownership intent.
- Ownership decisions still require reviewer judgment for edge cases.

## Related
- `docs/05-system-architecture/02-crate-architecture.md`
- `docs/08-development/02-testing-strategy.md`
- `docs/08-development/03-adapter-development.md`
- `docs/08-development/04-contributing.md`
