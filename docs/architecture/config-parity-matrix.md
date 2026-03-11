# Config Parity Matrix

Scope: configuration parity matrix freeze.

| Command | Rust Status | Parity Status | Notes |
|---|---|---|---|
| `config` (root) | implemented | parity-complete | file-backed listing baseline |
| `config get` | implemented | parity-complete | not-found/streams/format coverage |
| `config set` | implemented | parity-complete | stdin fallback and write safety covered |
| `config unset` | implemented | parity-complete | existing/missing/malformed coverage |
| `config clear` | implemented | parity-complete | non-empty/empty/missing/write-failure coverage |
| `config reload` | implemented | parity-complete | success/malformed/missing coverage |
| `config export` | implemented | parity-complete | path-required and text rejection aligned |
| `config load` | implemented | parity-complete | valid/malformed/duplicate/path handling covered |
