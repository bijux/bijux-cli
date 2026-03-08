# Advanced Semantics Quarantined Surfaces

This page lists advanced semantics families that remain quarantined from default runtime UX and core kernel semantics.

## Quarantined families

- distributed control-plane and federation modeling
- AI/operator-assist workflow modeling
- workflow product abstraction modeling
- dataset/catalog semantic modeling
- cost optimization modeling

## Why quarantined

- No concrete default user-facing execution path (`user_facing_path=false`).
- No direct test authority in core runtime (`direct_test=false`).
- No fixture-backed executable ownership (`example_or_fixture=false`).
- Must satisfy lifecycle policy `expire-or-graduate` with owner and target date.

## Graduation requirements

- Concrete user path added with explicit contract tests.
- Fixture-backed examples and deterministic behavior checks.
- Reclassification from `speculative` to a retained category in governance policy.
