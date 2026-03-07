# Test trust coverage report

## Runtime test surfaces

- runtime test files (`crates/bijux-dag-runtime/tests/*.rs`): `69`
- test trust catalog fixture: present
- trust classes covered: semantic, adversarial, failure, replay, scheduler, policy, cache, artifact, cancellation, state machine, recovery, import/export, node execution, scheduler determinism

## Risk notes

- trust catalog requires continuous pruning to prevent redundant overlap
