---
title: App Integration Scenario
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# App Integration Scenario

This repository evidence scenario checks that documentation and release
governance keep two extension boundaries distinct: an officially mounted app
and a user-installed plugin. It does not execute either fixture or establish
that the fixture files are reusable installation templates.

## Governed Inputs

- `evidence/authoring/examples/app-integration/mock-official-app.mount.json`
  records the official-app classification fields consumed by this evidence
  scenario.
- `evidence/authoring/examples/app-integration/mock-plugin.manifest.json`
  represents an independently installed plugin manifest.
- `configs/dag/release/app_integration_scenario.json` records the commands and
  expected delivery behavior used by repository validation.

The official-app evidence fixture predates the complete
`ProductMountDescriptor` JSON shape. It is intentionally retained as governed
release evidence and must not be passed to `bijux apps validate-manifest` or
copied into `.bijux/apps/`. Generate the current schema with
`bijux apps schema --json` and use the
[App Integration Guide](app-integration-guide.md) for a valid descriptor.

## Verification

```bash
bijux apps list --json
bijux apps which atlas --json
bijux apps doctor atlas --json
bijux plugins list --json
bijux plugins doctor --json
```

The app inventory must identify the selected official mount and its source.
App doctor must diagnose the official mounted executable without executing
unrelated plugins. The plugin inventory and doctor remain separately
addressable. JSON output is required because the scenario protects field-level
delivery contracts; human rendering may evolve without changing those fields.

## Failure Meaning

A missing official app usually indicates distribution metadata, override
discovery, disablement, or entrypoint resolution failure. A missing plugin
indicates installation registry or manifest validation failure. If both
inventories fail, inspect the shared state root and output envelope before
changing either integration. Keeping these diagnoses separate prevents a
plugin failure from being presented as a broken official product mount.
