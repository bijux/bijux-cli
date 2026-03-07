# App crate boundary decision

## Decision

`bijux-dag-app` remains a single crate for now.

## Rationale

- Command orchestration and output formatting share tight response-model contracts.
- Splitting into `runbook` or standalone `commands` crates would currently duplicate
  graph/runtime wiring and increase compatibility burden.
- Current module split (`commands`, `format`, `read`, `write`, `explain`, `graph`,
  `cache`, `replay`, `migrate`) provides boundary clarity inside one crate.

## Trigger to revisit

Revisit split when one of these is true:

- command families need independent release cadence
- app crate compile times materially regress due to command growth
- API stability policy requires separate crate-level versioning for command surfaces
