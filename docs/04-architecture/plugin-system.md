# Plugin System

The plugin system is a runtime-managed lifecycle, not a loose extension convention.

## Core Claim

Plugins are installed from `plugin.manifest.json`, persisted in the plugin registry, and validated again against current runtime conditions when the runtime needs to trust them.

## Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Discovered
    Discovered --> Validated
    Validated --> Installed
    Installed --> Enabled
    Installed --> Disabled
    Enabled --> Disabled
    Disabled --> Enabled
    Enabled --> Broken
    Installed --> Broken
    Enabled --> Incompatible
```

```mermaid
flowchart TD
    Manifest[plugin.manifest.json] --> Validate[Parse and validate manifest]
    Validate --> Entrypoint[Check current entrypoint availability]
    Entrypoint --> Registry[Write registry record]
    Registry --> Inspect[Info, inspect, where, and explain]
    Registry --> Doctor[Check, doctor, and load diagnostics]
```

## Current Plugin Model

The current plugin model includes:

- a versioned manifest contract
- namespace ownership rules
- alias rules
- compatibility ranges
- trust labels
- local entrypoint resolution
- routed execution for installed plugin namespaces and routed aliases
- reserved-name and alias collision checks, including official product
  namespaces and alias reuse of blocked names
- install, info, inspect, check, enable, disable, uninstall, scaffold, doctor,
  reserved-names, where, explain, and schema commands

## What The Runtime Checks Now

The runtime checks more than stored registry state.

Current health evaluation includes:

- manifest validity
- namespace and alias collision safety
- compatibility range against the current runtime semver
- manifest drift after install
- delegated or Python entrypoint availability through `module:callable`
  contracts
- external executable existence and executability

## Namespace And Alias Safety

```mermaid
flowchart LR
    Namespace[Requested namespace] --> Policy[Reserved namespace policy]
    Alias[Requested aliases] --> Policy
    Policy --> Conflict[Collision checks]
    Conflict --> Install[Registry update or rejection]
```

```mermaid
sequenceDiagram
    participant U as User
    participant I as Install command
    participant V as Validator
    participant G as Registry

    U->>I: install manifest
    I->>V: parse and validate
    V->>V: namespace, alias, compatibility, entrypoint checks
    V->>G: write validated record
    G-->>U: installed or rejected
```

## Honest Limits

The plugin system does not currently claim:

- sandboxing
- a stable in-process native ABI
- that every plugin kind shares one identical stdout and rendering model

The plugin system does claim that installation and health reporting are real runtime responsibilities, not documentation-only promises.
Installed plugin namespaces are executable runtime routes, but they still obey
their declared kind and current registry health.

## Why The Design Is Conservative

The plugin system touches user state and local files. Conservative validation is cheaper than recovering from a broken registry or a plugin that was accepted on weak assumptions.
