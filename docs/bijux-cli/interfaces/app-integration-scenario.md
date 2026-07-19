---
title: App Integration Scenario
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-05-01
---

# App Integration Scenario

This scenario demonstrates one official app integration path and one plugin
integration path through the root `bijux` runtime.

## Example Assets

- `evidence/dag/authoring/examples/app-integration/mock-official-app.mount.json`
- `evidence/dag/authoring/examples/app-integration/mock-plugin.manifest.json`

## Root CLI Flow

```bash
bijux apps list --json
bijux apps which atlas --json
bijux apps doctor atlas --json
bijux plugins list --json
bijux plugins doctor --json
```

The same command flow is enforced by the distribution delivery contract at
`configs/dag/release/app_integration_scenario.json`.
