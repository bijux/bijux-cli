.PHONY: test lint security fmt golden compat sanity dep-guard

test:
	cargo test --workspace

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all

security:
	cargo audit

golden:
	./scripts/golden.sh

compat:
	cargo run -p bijux_cli -- dag compat

sanity:
	./scripts/sanity.sh

dep-guard:
	./scripts/dep_guard.sh
