# Artifact Store Capability Matrix

| capability | filesystem store | object store model |
|---|---|---|
| write artifact payload | implemented | modeled |
| read artifact payload | implemented | modeled |
| integrity verify | implemented | modeled |
| lineage traversal | implemented | modeled |
| retention planning | implemented | modeled |
| runtime-backed execution | implemented | not implemented |

Notes:
- Runtime source-of-truth currently implements filesystem storage semantics.
- Object-store surface remains declared capability only and must not be presented as implemented runtime behavior.

