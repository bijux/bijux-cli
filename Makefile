.PHONY: test lint security fmt golden compat

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
	cargo run -p bijux_dag_cli -- compat
