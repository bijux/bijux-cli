# Stable Schema Compatibility Review Checklist

1. Confirm schema authority path under `configs/schema/` is correct and versioned.
2. Confirm command-family mapping is declared in `configs/policy/json_output_governance.json`.
3. Confirm minimal and maximal examples exist under `evidence/operator/examples/stable_json/<schema>/`.
4. Confirm lockstep test marker exists in `crates/bijux-dev-dag/tests/json_output_governance_contracts.rs`.
5. Confirm generated inventories are refreshed:
- `docs/reports/foundation/JSON_COMMAND_SCHEMA_INVENTORY_REPORT.md`
- `docs/reports/foundation/SCHEMA_COMMAND_TEST_INVENTORY_REPORT.md`
6. Confirm gap reports stay zero:
- `docs/reports/foundation/schema_without_example_output_report.md`
- `docs/reports/foundation/commands_without_json_lockstep_report.md`
7. Confirm schema registry and stable command registry pages are refreshed.
8. Confirm release gate suite still includes JSON output governance contract tests.
