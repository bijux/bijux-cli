# Evidence Commands Not Exercised in `make test-release`

Non-release-critical commands intentionally excluded from `make test-release`.

- `verify evidence-schema`
- `verify evidence-registry`
- `verify evidence-authoring`
- `verify evidence-drift`
- `verify evidence-foundation`
- `verify evidence-compare`

Sources:
- `configs/policy/evidence_rationalization_policy.json`
- `make/root.mk`
