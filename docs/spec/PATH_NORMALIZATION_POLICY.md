# Path normalization policy

Applies to DAG output paths, input materialization paths, and artifact index paths.

Rules:
- Paths must be relative.
- Absolute paths are invalid.
- Parent traversal (`..`) is invalid.
- Backslash separators are normalized to slash separators for canonical comparison.
- Canonicalization must preserve deterministic ordering independent of OS path separator representation.
