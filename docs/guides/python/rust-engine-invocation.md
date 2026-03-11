# Invoking the Rust Command Engine from Python

The Python facade always targets the Rust-backed runtime.

Resolution order:

1. `BIJUX_BIN` environment override.
2. `bijux` in `PATH`.
3. `bijux-rs` in `PATH` (compatibility alias, deprecated).

If no runtime binary is found, the facade raises `PlatformWheelUnavailable`.
