# Testing And Evidence

## Goal

Tie claims to evidence. This repository should not rely on confidence language
when current tests or maintainer checks can answer the question directly.

```mermaid
flowchart TD
    A[Code or doc change] --> B[Targeted tests]
    B --> C[Workspace checks]
    C --> D[Maintainer checks when needed]
    D --> E[Reviewable evidence]
```

```mermaid
stateDiagram-v2
    [*] --> Changed
    Changed --> Tested
    Tested --> Checked
    Checked --> Reviewed
    Reviewed --> Merged
```

## Evidence Rules

- do not describe work as done if the relevant tests are red or missing
- generated artifacts are disposable outputs, not repository truth
- prefer live tests and maintainer commands over hand-maintained status prose
- if a public behavior changed, update the tests and docs that describe it

## Common Verification Commands

```bash
cargo test --locked --workspace
python3 -m pytest crates/bijux-cli-python/tests/python
bijux dev cli status --format json --no-pretty
bijux dev cli parity --format json --no-pretty
bijux dev cli docs-audit --format json --no-pretty
make docs-check
```

## Review Rule

A convincing change explanation should answer:

- what changed
- what evidence proves it
- what still remains uncertain, if anything

## Honest Limit

Passing tests do not make every architectural claim true by themselves. Tests
and docs have to agree, and unsupported claims should still be stated as
unsupported.

## Read Next

Continue to [Release And Compatibility](release-and-compatibility.md).
