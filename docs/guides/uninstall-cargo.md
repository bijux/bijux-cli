# Uninstall for Cargo Users

Remove both canonical and compatibility package channels to avoid stale binaries.

```bash
cargo uninstall bijux-cli
cargo uninstall bijux
```

Then verify no cargo-owned binary remains:

```bash
bijux cli doctor
```

If `bijux` still resolves, it is coming from another channel (for example pip).

