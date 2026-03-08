# Awkward Fixture Families Cleanup Report

This report highlights fixture families that still create unnecessary maintenance weight.

## Cleanup targets
1. Duplicated fixtures across graph/run/artifact/replay suites.
2. Broad multi-purpose fixtures that reduce contract precision.
3. Legacy fixtures still discoverable in default smoke flows.

## Cleanup direction
- Promote canonical fixtures as default smoke inputs.
- Keep stress and corrupt fixtures explicit and targeted.
- Remove orphan fixtures and enforce fixture governance checks.
