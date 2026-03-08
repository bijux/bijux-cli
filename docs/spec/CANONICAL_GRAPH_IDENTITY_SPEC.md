# Canonical Graph Identity Specification

## Purpose

Define the canonical, deterministic identity for a graph as the hash of canonical graph JSON.

## Identity definition

- Canonical algorithm version: `bijux-dag-canonical/v1`
- Hash algorithm: `sha256`
- Graph identity value: `sha256(canonical_graph_json_bytes)`

## Canonicalization rules

1. Parse graph with strict schema (`serde` unknown fields rejected).
2. Normalize accepted spec aliases to `bijux-dag/v0.1`.
3. Normalize path separators to `/` for output paths.
4. Normalize Unicode identity text fields using NFC.
5. Sort nodes by `id`.
6. Sort node inputs.
7. Sort node outputs by output `name`.
8. Sort effects by stable effect order.
9. Sort env allowlist and tags.
10. Sort edges by `(from.node_id, from.port, to.node_id, to.port)`.
11. Sort JSON object keys in graph inputs and resolved params recursively.
12. Treat explicit zero resources (`cpu=0`, `mem_mb=0`) as absent resources.

## Semantic vs non-semantic changes

Non-semantic (must not change identity):
- JSON key ordering
- YAML key ordering after YAML->JSON normalization
- Whitespace-only formatting changes
- Comment-only source changes when comments are stripped before strict parse
- Edge list permutation

Semantic (must change identity):
- Node command/params changes
- Dependency edge topology changes
- Resource specification changes

## Backend independence

Graph identity is derived from canonical graph content only; runtime backend execution path is outside identity derivation.

## Contract surfaces

- Core API: `Graph::graph_id`, `Graph::graph_fingerprint_explain`
- CLI: `dag hash graph`, `dag fingerprint --explain`
