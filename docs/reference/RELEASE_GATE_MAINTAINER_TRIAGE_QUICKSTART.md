# Release Gate Maintainer Triage Quick-Start

1. Identify failing gate and owning area from owner/escalation matrix.
2. Reproduce failure locally with same `make` target.
3. Classify as product bug, policy drift, docs drift, or generated output drift.
4. Require targeted regression test or contract update with each fix.
5. Close loop by re-running affected gate and adjacent release-critical gates.
