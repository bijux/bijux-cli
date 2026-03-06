.PHONY: test lint security fmt golden compat sanity dep-guard

all: test

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
	cargo run -p bijux-dev-dag -- golden

compat:
		cargo run -p bijux-dag-cli -- dag compat

sanity:
	cargo run -p bijux-dev-dag -- sanity

dep-guard:
	cargo run -p bijux-dev-dag -- dep-guard

public-api:
	cargo run -p bijux-dev-dag -- public-api

ci:
	cargo run -p bijux-dev-dag -- ci
