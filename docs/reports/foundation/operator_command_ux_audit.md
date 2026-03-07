# Operator Command UX Audit

Audited command wording families:

- `validate`, `run`, `replay`, `diff`, `why-rerun`
- `runs inspect`, `runs explain-failure`, `trace-artifact`

Consistency checks:

- commands use explicit nouns for evidence (`run`, `artifact`, `cache`)
- diagnostic commands return root-cause summaries in JSON mode
- imported runs are labeled as `origin: imported` in human inspection output

