# bijux-cli-python

`bijux-cli-python` is the Python package and Rust bridge for the `bijux-cli` distribution.

## Scope

- Build the `bijux_cli_py._native` extension module with PyO3.
- Ship the Python wrapper package in `python/bijux_cli_py`.
- Expose compatibility helpers needed by the Python install path.
- Fall back to subprocess execution when the native extension is unavailable.

## Layout

- `src/`: Rust bindings, compatibility helpers, and conversion code.
- `python/bijux_cli_py`: Python wrapper package, subprocess fallback, and console entrypoint.
- `tests/*.rs`: Rust-side bridge and compatibility tests.
- `tests/python`: packaging, runtime parity, and runtime resilience tests.
- `tests/fuzz/bridge_conversion_minimized_cases`: retained JSON bridge regression samples.

## Boundary

- This crate does not define command routing, output contracts, or runtime command law.
- Behavior that belongs to `bijux-cli` stays in `bijux-cli`; this crate only bridges to it.
