# Contributing

Define contribution workflow, review standards, and merge readiness for repository changes.

Contributors need one clear path from idea to merged change with explicit quality gates.

## Explanation
Contribution workflow:
1. open a focused change with clear scope.
2. implement code/docs updates with domain-correct placement.
3. run required local checks relevant to changed areas.
4. submit for review with explicit summary of guarantees and limitations affected.
5. address feedback and merge after gate criteria pass.

Developer setup before first contribution:
1. install pinned Rust toolchain and ensure `cargo` is available.
2. clone repository and verify baseline commands execute locally.
3. run formatting/linting/tests expected for touched scope.
4. confirm docs/spec terminology alignment for any contract text edits.

Pull request expectations:
- one change intent per PR whenever possible.
- clear commit history using conventional commit style.
- explicit mention of behavior changes, contract changes, and migration impact.

Review gates:
- correctness: behavior and contracts are coherent and test-backed.
- clarity: names, file locations, and docs are understandable long-term.
- safety: sensitive operations and trust boundaries are respected.
- maintainability: no avoidable duplication, coupling, or stale scaffolding.

Final quality pass guidance:
- keep prose concise while preserving contract precision.
- remove stale claims or speculative statements.
- keep examples realistic and aligned with current command surfaces.
- ensure every guarantee has an explicit boundary or limitation nearby.
- avoid link dumps; include only highest-signal next references.
- ensure opening lines answer the reader's immediate question.
- prefer direct mechanics over abstract policy language.

Documentation contribution standards:
- docs must use required section template.
- claims must distinguish guarantees vs limitations.
- terminology must align with introduction/specification vocabulary.
- avoid governance noise in user-facing guides.
- every section must teach at least one non-obvious fact.
- no section should exist only to satisfy formatting symmetry.
- guarantees must be explicit, bounded, and falsifiable.
- limitations must be concrete and operationally useful.
- distinguish guarantees from goals and backend-dependent behavior.
- include realistic examples and expected outcomes.
- include at least one failure-oriented example where confusion risk is high.
- include common wrong assumptions when over-interpretation is likely.

Commit message guidelines:
- use meaningful conventional prefixes (`feat`, `fix`, `docs`, `refactor`, `test`, `chore`).
- subject line must describe durable intent, not temporary process step.
- avoid ambiguous wording such as "misc", "cleanup", or "phase".

## Examples
```bash
git commit -m "docs(specification): define replay and diff semantics contracts"
```

```text
PR summary template:
- scope
- behavior change
- contract impact
- validation evidence
```

## Guarantees
- Contributors and reviewers share a single merge workflow and quality gate.
- Commit and PR requirements emphasize durable, audit-friendly change history.
- Documentation contributions are held to explicit contract-quality expectations.
- Includes actionable developer setup and final quality-pass guidance.

## Limitations
- This guide does not replace technical design judgment for complex architecture changes.
- Required checks vary by change scope and repository tooling evolution.
- Merge approval remains a maintainer responsibility.

## Related
- `docs/08-development/01-repository-structure.md`
- `docs/08-development/02-testing-strategy.md`
- `docs/07-operations/01-ci-integration.md`
- `docs/06-specification/01-dag-model.md`
