# Cache Evidence

Use cache evidence when the claim is about reuse, invalidation, replay, or
corruption handling.

## What Lives Here

- `scenarios/`
- `corrupt/`
- `replay/`

## Boundary

- Cache evidence owns cache and replay fixture truth under `evidence/cache/**`.
- Battle evidence may consume cache evidence, but must not redefine cache fixtures under `evidence/battle/**`.
