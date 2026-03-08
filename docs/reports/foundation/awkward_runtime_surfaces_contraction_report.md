# Awkward Runtime Surfaces Contraction Report

This report tracks runtime and runtime-adjacent surfaces that still look broader than supported execution reality.

## Remaining contraction targets
1. Runtime modules with adapter-facing names but internal-only semantics.
2. Helper surfaces that expose internal scheduling details as if stable API.
3. Re-export patterns that leak quarantine namespaces into broad discoverability.
4. Mixed ownership boundaries across runtime and governance helper crates.

## Contraction direction
- Keep local deterministic execution as the stable center.
- Keep speculative and experimental scopes quarantined and explicitly owned.
- Keep public runtime entrypoints minimal and test-gated.
