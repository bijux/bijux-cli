# Error Codes

This catalog tracks stable public code IDs.

## Taxonomy source
- Registry: `configs/policy/error_codes.json`
- Contract: `docs/spec/ERROR_CONTRACT.md`
- Taxonomy: `docs/spec/ERROR_TAXONOMY.md`

## Stable codes
- `BJX-PARSE-001` (`parse`) Input is not valid DAG JSON.
- `BJX-SCHEMA-001` (`schema`) JSON shape violates schema contract.
- `BJX-VALIDATION-001` (`validation`) Semantic graph validation failed.
- `BJX-CONFIG-001` (`config`) Invalid configuration input.
- `BJX-POLICY-001` (`policy`) Policy denied requested behavior.
- `BJX-EXEC-001` (`execution`) Node execution failed.
- `BJX-IO-001` (`io`) Filesystem or artifact I/O failed.
- `BJX-REPLAY-001` (`replay`) Replay mismatch against recorded artifacts.
- `BJX-CACHE-001` (`cache`) Cache contract or proof mismatch.
- `BJX-COMPAT-001` (`compatibility`) Compatibility contract violation.
- `BJX-INTERNAL-001` (`internal`) Unexpected internal failure path.

## Change policy
New public codes require:
1. Registry update.
2. Contract/reference docs update.
3. Error tests update.
