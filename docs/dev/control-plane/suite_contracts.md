# Control-plane suite contracts

## Suite command contract

Suite groups (`checks`, `tests`, `contracts`, `repo`, `docs`, `release`) must expose:

- `run` with domain and filter flags
- `list` with stable machine-readable suite metadata
- `explain` for one suite id

## Run behavior contract

- Blocking mode: any failed suite returns command error.
- Advisory mode (`--advisory`): failed suites are reported but command returns success.
- Explanation mode (`--why`): suite selection report includes selected and skipped suite IDs with reason categories.

## Foundation super-suite contract

The `foundation` command executes `checks`, `tests`, `contracts`, `repo`, and `docs` with shared filtering and policy flags.

## Machine-readable schemas

- `configs/schema/dev-control/command_report.schema.json`
- `configs/schema/dev-control/suite_selection_report.schema.json`
- `configs/schema/control_plane/evidence_suite_report.schema.json`

## Evidence verification suites

Evidence verification is first-class and must remain visible in control-plane usage and CI:

- `verify evidence-schema`
- `verify evidence-registry`
- `verify evidence-authoring`
- `verify evidence-battle`
- `verify evidence-cache`
- `verify evidence-replay`
- `verify evidence-compat`
- `verify evidence-fault`
- `verify evidence-perf`
- `verify evidence-compare`
- `verify evidence-consumers`
- `verify evidence-drift`
- `verify evidence-release-set`
- `verify evidence-foundation`
