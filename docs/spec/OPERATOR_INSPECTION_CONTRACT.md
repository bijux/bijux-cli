# Operator inspection contract

## Scope
Defines stable operator inspection surfaces, output semantics, and failure classification behavior for run inspection commands.

## Stable commands

- `dag runs inspect`
- `dag runs show`
- `dag runs timeline`
- `dag runs tree`
- `dag runs explain-failure`
- `dag runs verify`
- `dag runs doctor`

## Integrity classification

Inspection outputs must classify run integrity using:

- `healthy`
- `incomplete`
- `corrupt`
- `unsupported`

## JSON schema surfaces

- `configs/schema/operator/run_inspect.schema.json`
- `configs/schema/operator/run_show.schema.json`
- `configs/schema/operator/run_timeline.schema.json`
- `configs/schema/operator/run_tree.schema.json`
- `configs/schema/operator/run_explain_failure.schema.json`
- `configs/schema/operator/run_doctor.schema.json`

## Human-readable surfaces

- `dag runs show` must provide a concise summary focused on run identity, status, integrity, and timing.
- `dag runs inspect` must remain concise while including retries, cache hits, and artifact counts.

## Timeline reconstruction requirements

Timeline reconstruction must include:

- execution ordering by start timestamp
- retry attempt information
- cache-hit markers
- coherence with trace timestamps

## Portability and context rules

- Inspection commands must work from explicit run roots without ambient repository state.
- Imported runs must remain inspectable and distinguishable in integrity/provenance reporting.

## Versioning and change policy
Any inspection output shape change requires schema updates, contract updates, and operator tests in the same change.
