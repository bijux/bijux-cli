# Install And Verify

## Goal

Choose one installation channel, verify the active runtime immediately, and
avoid carrying ambiguous installs into later automation.

```mermaid
flowchart LR
    A[Choose one channel] --> B[Install bijux-cli]
    B --> C[Run verification commands]
    C --> D{Healthy result?}
    D -->|Yes| E[Continue to command usage]
    D -->|No| F[Resolve path or install conflicts]
```

```mermaid
sequenceDiagram
    participant U as User
    participant S as Shell
    participant B as bijux
    U->>S: install bijux-cli
    U->>B: bijux version
    B-->>U: runtime identity
    U->>B: bijux cli paths
    B-->>U: active binary and state paths
    U->>B: bijux cli doctor
    B-->>U: install health report
```

## Recommended Channels

- `cargo install --locked bijux-cli`
- `python -m pip install --upgrade bijux-cli`
- `pipx install bijux-cli`

Pick one. Using several channels at once is possible, but it increases the
chance of path ambiguity and stale wrappers.

## Verify Immediately

Run:

```bash
bijux version
bijux cli paths
bijux cli doctor
```

These commands answer three different questions:

- does `bijux` resolve at all
- which binary and state paths are active
- is the current install healthy or shadowed by another channel

## Honest Rule

Do not treat an install as complete until `bijux cli doctor` is clean enough
for your intended use. A command existing in `PATH` is not the same as a sound
runtime setup.

## Read Next

Continue to [Installation And Recovery](installation-and-recovery.md).
