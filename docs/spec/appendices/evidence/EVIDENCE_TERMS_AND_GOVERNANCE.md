# Evidence, Proof, Verification, and Governance

Canonical vocabulary and meanings are defined in:
- `docs/spec/EVIDENCE_GLOSSARY.md`

Operational model:
1. Evidence assets are stored under governed roots and tracked by the ledger/registry.
2. Verification commands (`bijux-dev-dag verify evidence-*`) evaluate policy and integrity contracts.
3. Proof surfaces aggregate verification outcomes for release and trust communication.
4. Governance policy classifies which evidence checks are release-critical versus advisory.

Lane behavior:
- Fast lane keeps advisory evidence non-blocking by default.
- Full lane executes the release-critical evidence command set and blocks on failure.
