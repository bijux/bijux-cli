# Operator Command Value Map

Primary operator flow:

1. `validate` -> confirm DAG correctness
2. `plan` -> inspect intended execution
3. `run` -> execute and persist run artifacts
4. `runs`/`inspect` -> inspect run history and diagnostics
5. `diff`/`replay` -> compare and replay behavior
6. `prove`/`verify` -> trust and reproducibility checks

Supporting flow:

- `capabilities` for backend/surface compatibility
- `export`/`import` for bundle portability workflows
