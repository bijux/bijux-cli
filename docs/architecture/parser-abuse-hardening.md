# Parser Abuse Hardening

Scope: tasks `401-420`.

Evidence:
- `crates/bijux-cli-routing/tests/parser_abuse.rs`
- `artifacts/status/parser_abuse_report.json`

Law:
- parser and routing must stay deterministic under malformed argv, ambiguity, alias pressure, and registration-order variation
- reserved and built-in namespace routes must not be hijacked by plugin namespace tokens
- parser abuse coverage is required before major release-completeness claims
