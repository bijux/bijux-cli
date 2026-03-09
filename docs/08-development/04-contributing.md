# Contributing

## Purpose
Define the documentation writing and review standards for all contributions.

## Context
This guide establishes the documentation foundation for bijux-dag. It sets writing philosophy, style boundaries, review expectations, and acceptance criteria so all docs remain clear, truthful, and durable.

## Explanation

### Documentation Foundation Standards (Tasks 1-20)

1. Documentation philosophy
- Truth over marketing.
- Precision over persuasion.
- Explanations must describe observed behavior and defined guarantees.

2. Tone
- Calm, technical, and direct.
- No hype language and no sales framing.

3. Writing style
- Short paragraphs.
- One idea per paragraph.
- Concrete examples before abstraction when possible.

4. Banned language
- Ban speculative claims presented as facts.
- Ban absolute performance claims without measured context.
- Ban vague phrases such as "world class", "next generation", or "revolutionary".

5. Required document structure
- Every document must use:
  - `Purpose`
  - `Context`
  - `Explanation`
  - `Examples`
  - `Guarantees`
  - `Limitations`
  - `Related`

6. Cross-linking rules
- Link only to directly related documents.
- Keep `Related` lists between 2 and 5 links.
- Avoid link dumps and circular over-linking.

7. Glossary ownership
- Terminology source of truth is `docs/01-introduction/05-terminology.md`.
- New terms must be added there before use elsewhere.

8. Example standards
- Examples must be realistic and minimal.
- Examples must show expected outcomes, not only commands.
- Avoid toy abstractions that hide constraints.

9. CLI example formatting
- Use fenced `bash` blocks.
- Include command intent in nearby prose.
- Include expected important output fields.

10. Code block style rules
- Every code fence includes a language identifier.
- Keep blocks focused to the smallest useful snippet.
- Do not include dead code or pseudocode presented as executable.

11. Diagram usage rules
- Diagrams are optional and used only for structural clarity.
- Diagrams must match current behavior and terminology.

12. When diagrams are allowed
- Allowed when relationships are hard to explain linearly.
- Not allowed when prose and short examples are sufficient.

13. When tables are allowed
- Allowed for command matrices, compatibility, and guarantees.
- Not allowed for narrative explanations.

14. Maximum document length guidance
- Default target: concise and readable in one sitting.
- Split a document when it becomes a topic bundle rather than a topic.
- Guidance threshold:
  - 0-120 lines: normal.
  - 121-220 lines: require explicit justification in review.
  - 221+ lines: split unless contract continuity demands a single file.

15. Example DAG style
- Use minimal deterministic DAG examples.
- Show dependency order explicitly.
- Include run and artifact implications.

16. Artifact example style
- Use concrete artifact paths.
- Show identity and provenance-relevant fields when applicable.

17. Naming rules for docs
- Lowercase kebab-case with numeric ordering prefix inside sections.
- Names must communicate topic intent, not process status.

18. Contributor writing checklist
- Uses required document structure.
- Defines guarantees and limitations explicitly.
- Uses canonical terminology.
- Includes realistic examples.
- Keeps links focused and non-redundant.
- Marks every non-obvious claim as either observed behavior or intended contract.

19. Maintainer review checklist
- Claims match current system behavior.
- No duplicated concept definitions across multiple docs.
- Guarantees are testable or contract-backed.
- Limitations are explicit and honest.
- Language is neutral and precise.
- Any moved/merged content preserves reader meaning without archaeology.

20. Documentation quality acceptance criteria
- Accurate: factual and implementation-aligned.
- Coherent: clear reading path and no contradiction.
- Minimal: no duplication and no governance noise in user-facing docs.
- Useful: includes practical examples and operational meaning.
- Durable: understandable two years later without archaeology.

### Enforceable Review Gates
Before merge, maintainers must answer all gates with `yes`:
1. Is this document necessary and non-duplicate?
2. Are guarantees and limitations explicit?
3. Are terms aligned with `05-terminology.md`?
4. Are examples concrete and executable or clearly marked illustrative?
5. Does this change improve or preserve the reading path?

### Contribution Workflow
1. Draft content inside the target document using the required structure.
2. Run a self-review against the contributor checklist.
3. Request maintainer review with highlighted guarantee and limitation changes.
4. Merge only after checklist completion.

## Examples
```bash
# Good example style: command + expected key output field
bijux-dag run --dag ./examples/minimal.dag.json
```

```text
Expected key output:
- run_id
- status
- artifact_count
```

## Guarantees
- All new documentation contributions follow one explicit quality standard.
- Reviews apply the same acceptance criteria across sections.
- Terminology remains consistent across the docs set.

## Limitations
- This guide defines quality and process, not runtime behavior.
- It does not replace product or spec contracts.

## Related
- `docs/01-introduction/03-design-principles.md`
- `docs/01-introduction/05-terminology.md`
- `docs/08-development/01-repository-structure.md`
- `docs/08-development/02-testing-strategy.md`
