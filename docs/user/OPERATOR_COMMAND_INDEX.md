# Operator Command Index

## Normative operator commands
- `dag runs list --root <runs_dir>`
- `dag runs show <run_id> --root <runs_dir>`
- `dag runs inspect <run_id> --root <runs_dir>`
- `dag runs tree <run_id> --root <runs_dir>`
- `dag runs timeline <run_id> --root <runs_dir>`
- `dag runs diff <run_a_dir> <run_b_dir> [--explain]`
- `dag runs verify <run_id> --root <runs_dir> [--deep]`
- `dag runs doctor <run_id> --root <runs_dir>`
- `dag runs explain-failure <run_id> --root <runs_dir>`
- `dag runs summary --root <runs_dir>`
- `dag runs compare <run_a> <run_b> --root <runs_dir>`
- `dag runs trend --root <runs_dir>`
- `dag runs failures --root <runs_dir>`
- `dag runs flakes --root <runs_dir>`

## Debug/internal commands
- `dag diff` (legacy alias)
- `dag status` (legacy alias)
- `dag verify` (legacy alias)
- `dag doctor` (legacy alias)

Legacy aliases are hidden from primary help and kept only for compatibility.
