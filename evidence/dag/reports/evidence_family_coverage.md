# Evidence Family Coverage

## Coverage Snapshot

| Family | Asset Count | Metadata File |
| --- | ---: | --- |
| cache | 10 | evidence/cache/metadata.json |
| replay (cache subfamily) | 4 | evidence/cache/metadata.json |
| compat | 8 | evidence/compat/metadata.json |
| fault | 3 | evidence/fault/metadata.json |

## Required Boundaries
- cache assets stay under evidence/cache/** and are validated by verify evidence-cache.
- replay fixtures stay under evidence/cache/replay/** and are validated by cache metadata replay outcomes.
- compat assets stay under evidence/compat/** and carry support decision metadata.
- fault assets stay under evidence/fault/** and carry expected fault class and system reaction.
