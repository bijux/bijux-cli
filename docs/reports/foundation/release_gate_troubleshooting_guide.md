# Release Gate Troubleshooting Guide

Standard triage pattern for gate failures:

1. Identify failing gate from CI summary.
2. Re-run gate locally with same make target.
3. Capture failing contract/test names and generated artifacts.
4. Determine whether failure is product logic, policy drift, or generated-doc drift.
5. Apply fix and re-run failing gate first, then `make test` and relevant full-lane gate.
6. Link root cause and fix evidence in PR description.
