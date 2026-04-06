# Parser Interesting Input Corpus

This directory contains `.txt` corpora replayed by
`tests/routing/parser/parser_fuzz_regressions.rs`.

- Each non-comment line is one space-delimited argv sequence.
- Lines usually start with `bijux`.
- Keep inputs adversarial but readable; this corpus is for broad parser coverage, not only minimized crashes.
