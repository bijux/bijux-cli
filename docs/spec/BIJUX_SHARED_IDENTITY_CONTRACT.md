# Bijux Shared Identity Contract

## Purpose
Define identity surfaces shared across `bijux-cli`, `bijux-dag`, `bijux-atlas`, and `bijux-dna`.

## Shared identities
- graph identity (`graph_id` canonical hash)
- run identity (`run_id` with ancestry semantics)
- artifact identity (`artifact_id` content + provenance)

## Rules
- product adapters may extend metadata but cannot redefine shared identity semantics.
- cross-product import/replay must preserve shared identities or emit explicit downgrade.
