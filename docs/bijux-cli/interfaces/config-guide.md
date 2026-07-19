---
title: Config Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Configuration Guide

`bijux` resolves configuration from typed layers. The effective value is not
necessarily the value in the nearest file, so diagnosis should start with the
runtime rather than with manual file inspection.

## Precedence

Configuration is applied from lowest to highest precedence:

1. global configuration
2. selected global profile
3. project `.bijux/config.toml` or `.bijux/config.json`
4. selected project profile
5. environment variables
6. explicit command arguments

The last defined value wins. A profile selects an overlay; it does not replace
the base document. Project files affect commands executed in that project and
must not silently rewrite global state.

## Inspect Before Changing

```bash
bijux config validate
bijux config explain cli.log_level
bijux config schema cli
bijux config docs cli
```

`validate` reports malformed or unsupported values. `explain` shows the
winning source for one key. `schema` is the machine-readable contract, while
`docs` is its operator rendering. Use `repair` only after reviewing the
diagnostic because repair may rewrite invalid persisted configuration.

Portable export and load commands are for moving supported values between
environments. They are not a secret transport. Sensitive values are redacted
from normal explanations and documentation; `--include-secrets` is an explicit
disclosure action and its output must be handled accordingly.

## Generated Authority

[`generated-config-reference.md`](generated-config-reference.md) is generated
from the same registry used by `bijux config schema`. If the checked-in page
and runtime output differ, regenerate the page and review the schema change;
do not maintain a handwritten parallel reference.
