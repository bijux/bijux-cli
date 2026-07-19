---
title: App Integration Scenario
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-05-01
---

# App Integration Scenario

This repository fixture proves two different extension boundaries through the
root `bijux` runtime: an officially mounted app and a user-installed plugin.
They share discovery and diagnostic conventions, but they do not share
ownership or release guarantees.

## Governed Inputs

- `evidence/dag/authoring/examples/app-integration/mock-official-app.mount.json`
  represents product mount metadata supplied by a Bijux distribution.
- `evidence/dag/authoring/examples/app-integration/mock-plugin.manifest.json`
  represents an independently installed plugin manifest.
- `configs/dag/release/app_integration_scenario.json` records the commands and
  expected delivery behavior used by repository validation.

These files are test fixtures, not templates for claiming an unofficial plugin
is an official app.

## Verification

```bash
bijux apps list --json
bijux apps which atlas --json
bijux apps doctor atlas --json
bijux plugins list --json
bijux plugins doctor --json
```

The app inventory must identify the selected mount and its source. App doctor
must diagnose the mounted executable without executing unrelated plugins. The
plugin inventory and doctor must remain usable when no official app is
selected. JSON output is used because the fixture protects field-level
delivery contracts; human rendering may evolve without changing those fields.

## Failure Meaning

A missing app usually indicates distribution metadata or mount discovery
failure. A missing plugin indicates installation registry or manifest
validation failure. If both inventories fail, inspect the shared state root and
output envelope before changing either integration. Keeping these diagnoses
separate prevents a plugin failure from being presented as a broken official
product mount.
