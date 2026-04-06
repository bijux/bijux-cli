# Quality And Change Management

This project tries to earn credibility by making architectural claims that can be checked.

## Principle

Documentation is not accepted as evidence by itself.

Architectural claims should be backed by one or more of:

- executable tests
- schema snapshots
- command snapshots where appropriate
- release and maintainer checks

## Evidence Flow

```mermaid
flowchart LR
    Claim[Architectural claim] --> Test[Executable tests]
    Claim --> Contract[Schema or contract]
    Claim --> Docs[Architecture docs]
    Test --> Confidence[earned confidence]
    Contract --> Confidence
    Docs --> Confidence
```

This diagram explains how the repository expects confidence to be earned.
Documentation contributes context, but executable tests and contract assets are
what keep an architectural claim reviewable instead of rhetorical.

```mermaid
sequenceDiagram
    participant Change as Code or docs change
    participant Tests as Tests and checks
    participant Review as Review
    participant Release as Release workflow

    Change->>Tests: verify behavior
    Tests->>Review: show evidence
    Review->>Release: accept or reject
    Release-->>Change: published or sent back
```

The sequence diagram shows the review loop around a change. It connects
implementation work, checks, review, and release so the quality model is tied
to the actual repository workflow.

## What Quality Means Here

Quality is not defined as “many documents” or “many reports.”

Quality means:

- public behavior is intentional
- legacy assumptions are removed when they are no longer true
- generated artifacts are disposable
- the repository does not pretend a temporary migration note is permanent architecture

## Change Rules

```mermaid
stateDiagram-v2
    [*] --> Proposed
    Proposed --> Implemented
    Implemented --> Verified
    Verified --> Documented
    Documented --> Released
```

This state diagram expresses the expected maturity path for a change. Shipping
is intentionally later than implementation so verification and documentation do
not become optional cleanup.

```mermaid
flowchart TD
    NewBehavior[New behavior] --> A{Public contract?}
    A -->|yes| B[Update tests, contracts, and docs]
    A -->|no| C[Update internal implementation only]
    B --> D[Review and release]
    C --> D
```

The final diagram shows how contract scope changes the required follow-up. New
public behavior must update tests, docs, and contracts together, while purely
internal changes can stay narrower.

## Documentation Rule

These architecture documents should stay:

- few in number
- current
- explicit about limits
- willing to say "not supported", "not public", or "not finished" when that is the truth

## Honest Status

This repository still has complexity and some historical drag. The answer is not to hide that drag under confident prose. The answer is to keep a small architecture canon, keep it accurate, and let the code and tests carry the rest.
