---
title: Cache Hardening Report
audience: maintainer
type: report
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Cache Hardening Report

## Purpose

This report records the repository surfaces that currently harden cache reuse,
corruption handling, and operator cache verification behavior.

## Guarded surfaces

- contract: `docs/spec/CACHE_CONTRACT.md`
- evolution model: `docs/spec/CACHE_EVOLUTION_MODEL.md`
- prune policy: `docs/spec/CACHE_PRUNE_POLICY.md`
- correctness ledger: `docs/reports/governance/CACHE_CORRECTNESS_COVERAGE.md`
- evidence metadata: `evidence/cache/metadata.json`
- corruption fixtures: `evidence/cache/corrupt/missing_meta.json`, `evidence/cache/corrupt/hash_mismatch.json`, `evidence/cache/corrupt/missing_manifest.json`, `evidence/cache/corrupt/unsupported_metadata_version.json`, `evidence/cache/corrupt/truncated_meta.json`, `evidence/cache/corrupt/missing_outputs_proof.json`
- warm/cold scenario: `evidence/cache/scenarios/warm_cold.json`
- runtime tests: `crates/bijux-dag-runtime/tests/cache_contracts.rs`, `crates/bijux-dag-runtime/tests/cache_evolution_contracts.rs`
- app tests: `crates/bijux-dag-app/tests/cache_evolution_contract.rs`
- maintainer tests: `crates/bijux-dev/tests/cache_hardening_contracts.rs`
- trust property: `tp_cache_integrity`

## Runtime proof boundary

- cache key explanation must remain stable for unchanged semantics
- proof completeness and metadata version checks must reject stale or missing
  entries
- operator cache routes must keep corruption visible through verification and
  explanation surfaces
