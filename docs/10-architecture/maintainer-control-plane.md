# Maintainer Control-Plane

The maintainer control-plane exists so repository diagnostics and release-facing checks do not have to distort the user-facing runtime.

## Separation

```mermaid
flowchart LR
    User[Runtime user] --> CLI[bijux]
    Maintainer[Maintainer] --> DEV[bijux-dev-cli]
    CLI --> Runtime[Runtime responsibilities]
    DEV --> Reports[Repository and release reports]
```

```mermaid
graph TD
    DEV[bijux-dev-cli] --> Contracts[contracts and inventories]
    DEV --> Health[repo and package health]
    DEV --> Routes[maintainer route registry]
    DEV --> Parity[maintainer parity views]
    DEV --> Runtime[uses bijux-cli contracts and helpers]
```

## Why This Crate Exists

Without a separate maintainer control-plane, the runtime binary would accumulate:

- repository-only diagnostics
- release-only gating logic
- documentation audits
- status inventories meant for maintainers rather than users

That would make the user runtime harder to reason about and harder to stabilize.

## Current Responsibilities

The control-plane currently covers:

- repository and package inventories
- maintainer route and contract views
- diagnostics around status, parity, and quality
- release-support logic

## Current Boundary

The maintainer crate depends on the runtime crate.

The runtime crate does not depend on the maintainer crate.

That direction is part of the architecture, not an accident.

## Release Coordination

```mermaid
sequenceDiagram
    participant Tag as Tag push
    participant CI as CI workflow
    participant Wait as release wait gate
    participant Publish as publish workflow

    Tag->>CI: trigger CI on tagged commit
    Tag->>Wait: trigger release workflow
    Wait->>CI: poll for matching CI result
    CI-->>Wait: success
    Wait->>Publish: allow publish
```

```mermaid
flowchart TD
    RepoState[Repository state] --> DEV
    RuntimeState[Runtime contracts and reports] --> DEV
    DEV --> MaintainerOutput[Maintainer output]
    MaintainerOutput --> ReleaseDecision[release or follow-up action]
```

## Honest Limit

The maintainer control-plane is not a public stability promise at the same level as the end-user CLI. It is valuable, tested, and important, but it is still a maintainer-oriented architecture surface.
