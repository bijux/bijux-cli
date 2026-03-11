# bijux-cli-python

Canonical Python bridge crate for `bijux-cli`.

## Responsibilities
- Own the Python extension module (`PyO3`) and wrapper package (`bijux_cli_py`).
- Delegate command execution to the Rust runtime surface.
- Expose compatibility helpers needed by the Python install path.

## Boundary
- Must not own command routing or output shaping laws.
- Must not reimplement runtime behavior that belongs to `bijux-cli`.
