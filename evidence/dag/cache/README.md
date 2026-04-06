# Cache Evidence

Purpose: cache correctness scenarios including warm/cold and corruption behavior.

Subdirectories:
- `scenarios/`
- `corrupt/`
- `replay/`

Boundary:
- Cache evidence owns cache and replay fixture truth under `evidence/cache/**`.
- Battle evidence may consume cache evidence, but must not redefine cache fixtures under `evidence/battle/**`.
