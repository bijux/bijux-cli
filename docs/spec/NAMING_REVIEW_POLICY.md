# Naming review policy

## Review gate

All changes introducing new normative names must satisfy:

- naming rules in `docs/spec/NAMING_GUIDELINES.md`
- glossary alignment in `docs/spec/TERMINOLOGY_GLOSSARY.md`
- audit mapping updates in `docs/architecture/naming_audit.md` when renaming

## Reviewer checklist

- name reflects stable behavior
- no transitional lifecycle wording in normative surfaces
- no banned marketing qualifiers in runtime module names
- tests/fixtures/docs updated with renamed symbols

## Ownership

- Runtime naming owner: runtime maintainers
- Artifact naming owner: artifact maintainers
- Governance naming owner: dev control-plane maintainers
