# Python vs Rust Parse Corpus

This corpus is derived from:
- `docs/reference/current-python/index.md`
- `docs/reference/current-python/plugin-command-and-lifecycle-behavior.md`
- existing Rust routing tests and snapshots.

Each line is an argv case intended for parser normalization and route resolution checks.

## Root and aliases
- `bijux status`
- `bijux doctor`
- `bijux version`
- `bijux repl --help`
- `bijux inspect`
- `bijux completion`

## Grouped cli
- `bijux cli status`
- `bijux cli paths`
- `bijux cli config get`
- `bijux cli config set`
- `bijux cli plugins list`
- `bijux cli plugins inspect`

## Grouped dev cli
- `bijux dev cli routes`
- `bijux dev cli registry`
- `bijux dev cli env`
- `bijux dev cli doctor`
- `bijux dev cli contracts`

## Legacy dev aliases
- `bijux dev routes`
- `bijux dev registry`
- `bijux dev env`
- `bijux dev doctor`
- `bijux dev contracts`

## Invalid and suggestions
- `bijux cli sttus`
- `bijux dev cli regstry`
- `bijux inspekt`

## Global flags before/after path
- `bijux --quiet --format json cli status`
- `bijux cli status --quiet --format json`
- `bijux --log-level debug --color never dev cli routes`
