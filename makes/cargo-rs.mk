# Rust quality gates and reports (artifact-scoped, no root pollution)

SHELL := /bin/bash

RS_ARTIFACT_ROOT ?= artifacts/rust
RS_RUN_ID ?= local

RS_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/target)
RS_NEXTEST_CACHE_DIR ?= $(RS_TARGET_DIR)/nextest
RS_PROFRAW_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/profraw)
RS_LLVM_PROFILE_FILE ?= $(RS_PROFRAW_DIR)/default_%m_%p.profraw

RS_FMT_REPORT ?= $(RS_ARTIFACT_ROOT)/fmt/$(RS_RUN_ID)/report.txt
RS_LINT_REPORT ?= $(RS_ARTIFACT_ROOT)/lint/$(RS_RUN_ID)/report.txt
RS_TEST_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest.log
RS_TEST_ALL_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-all.log
RS_AUDIT_REPORT ?= $(RS_ARTIFACT_ROOT)/audit/$(RS_RUN_ID)/report.txt
RS_COVERAGE_DIR ?= $(RS_ARTIFACT_ROOT)/coverage/$(RS_RUN_ID)
RS_LCOV_FILE ?= $(RS_COVERAGE_DIR)/lcov.info
RUST_PUBLISH_PACKAGES ?= bijux-cli bijux-cli-evidence bijux-cli-python bijux-dev-cli
RUST_PUBLISH_DRY_RUN ?= 1
RUST_PUBLISH_ALLOW_DIRTY ?= 0
RUST_PUBLISH_REGISTRY ?= crates-io

CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always

NEXTEST_PROFILE ?= default
NEXTEST_STATUS_LEVEL ?= all
NEXTEST_FINAL_STATUS_LEVEL ?= all
NEXTEST_SHOW_PROGRESS ?= counter
# Default fast-lane exclusions: known tests that consistently exceed 10 seconds.
# Override with NEXTEST_FILTER_EXPR to run a custom selection.
NEXTEST_SLOW_EXCLUDE_EXPR ?= not ( \
	test(/repo_health_exposes_stale_generated_artifact_detection/) or \
	test(/repo_docs_maintenance_crate_health_json_and_text_contracts/) or \
	test(/repo_health_json_contracts_are_stable/) or \
	test(/repo_text_heads_match_snapshots/) or \
	test(/executes_dev_cli_namespace_commands/) or \
	test(/all_command_groups_build_expected_top_level_keys/) \
)

define rs_require_tool
	@command -v $(1) >/dev/null 2>&1 || { \
		echo "$(1) is required but not installed"; \
		exit 1; \
	}
endef

define rs_nextest_summary
	summary_line=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$(1)" | grep 'Summary \[' | tail -n 1 || true); \
	printf '\033[1;36m%s\033[0m %s\n' "nextest-summary:" "$${summary_line:-unavailable}"
endef

.PHONY: fmt-rs lint-rs test-rs test-all-rs coverage-rs audit-rs publish-rs

fmt-rs:
	@mkdir -p "$(dir $(RS_FMT_REPORT))"
	@printf '%s\n' "run: cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml"
	@set -o pipefail; \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml 2>&1 | tee "$(RS_FMT_REPORT)"

lint-rs:
	@mkdir -p "$(dir $(RS_LINT_REPORT))"
	@printf '%s\n' "run: cargo clippy --workspace --all-targets --all-features --locked -- -D warnings"
	@set -o pipefail; \
	CLIPPY_CONF_DIR="configs/rust" \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings 2>&1 | tee "$(RS_LINT_REPORT)"

test-rs:
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(dir $(RS_TEST_REPORT))" "$(RS_PROFRAW_DIR)"
	@status=0; \
	filter_expr="$${NEXTEST_FILTER_EXPR:-$(NEXTEST_SLOW_EXCLUDE_EXPR)}"; \
	LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo nextest run \
		--workspace \
		--config-file configs/rust/nextest.toml \
		--user-config-file none \
		--profile "$(NEXTEST_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		--show-progress "$(NEXTEST_SHOW_PROGRESS)" \
		$${filter_expr:+-E "$${filter_expr}"} \
		2>&1 | tee "$(RS_TEST_REPORT)" || status=$$?; \
	$(call rs_nextest_summary,$(RS_TEST_REPORT)); \
	test $$status -eq 0

test-all-rs:
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(dir $(RS_TEST_ALL_REPORT))" "$(RS_PROFRAW_DIR)"
	@status=0; \
	LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo nextest run \
		--workspace \
		--run-ignored all \
		--retries 0 \
		--config-file configs/rust/nextest.toml \
		--user-config-file none \
		--profile "$(NEXTEST_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		--show-progress "$(NEXTEST_SHOW_PROGRESS)" \
		$${NEXTEST_FILTER_EXPR:+-E "$${NEXTEST_FILTER_EXPR}"} \
		2>&1 | tee "$(RS_TEST_ALL_REPORT)" || status=$$?; \
	$(call rs_nextest_summary,$(RS_TEST_ALL_REPORT)); \
	test $$status -eq 0

coverage-rs:
	$(call rs_require_tool,cargo-llvm-cov)
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(RS_COVERAGE_DIR)" "$(RS_PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	CARGO_LLVM_COV_TARGET_DIR="$(RS_TARGET_DIR)" \
	NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo llvm-cov nextest \
		--workspace \
		--all-features \
		--run-ignored all \
		--retries 0 \
		--config-file configs/rust/nextest.toml \
		--user-config-file none \
		--profile "$(NEXTEST_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		--show-progress "$(NEXTEST_SHOW_PROGRESS)" \
		--lcov --output-path "$(RS_LCOV_FILE)"
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	CARGO_LLVM_COV_TARGET_DIR="$(RS_TARGET_DIR)" \
	cargo llvm-cov report

audit-rs:
	$(call rs_require_tool,cargo-deny)
	$(call rs_require_tool,cargo-audit)
	@mkdir -p "$(dir $(RS_AUDIT_REPORT))"
	@set -o pipefail; { \
		echo "run: cargo deny check --config configs/rust/deny.toml"; \
		CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo deny check --config configs/rust/deny.toml; \
		echo; \
		echo "run: cargo audit"; \
		CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo audit; \
	} 2>&1 | tee "$(RS_AUDIT_REPORT)"

publish-rs:
	@set -euo pipefail; \
	if [ -z "$(RUST_PUBLISH_PACKAGES)" ]; then \
		echo "RUST_PUBLISH_PACKAGES is empty; nothing to publish"; \
		exit 1; \
	fi; \
	dry_run_flag=""; \
	if [ "$(RUST_PUBLISH_DRY_RUN)" = "1" ]; then \
		dry_run_flag="--dry-run"; \
	fi; \
	allow_dirty_flag=""; \
	if [ "$(RUST_PUBLISH_ALLOW_DIRTY)" = "1" ]; then \
		allow_dirty_flag="--allow-dirty"; \
	fi; \
	for pkg in $(RUST_PUBLISH_PACKAGES); do \
		echo "→ cargo publish -p $$pkg --registry $(RUST_PUBLISH_REGISTRY) $$dry_run_flag"; \
		CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
		cargo publish \
			--locked \
			--registry "$(RUST_PUBLISH_REGISTRY)" \
			-p "$$pkg" \
			$$allow_dirty_flag \
			$$dry_run_flag; \
	done

##@ Rust
fmt-rs: ## Run Rust format checks (artifact-scoped)
lint-rs: ## Run Rust clippy checks with -D warnings (artifact-scoped)
test-rs: ## Run Rust nextest fast suite and skip known >10s tests by default
test-all-rs: ## Run Rust nextest all-features + ignored suite (artifact-scoped)
coverage-rs: ## Run Rust llvm-cov via nextest and emit lcov/report (artifact-scoped)
audit-rs: ## Run cargo-deny and cargo-audit (artifact-scoped)
publish-rs: ## Publish Rust crates (dry-run by default; set RUST_PUBLISH_DRY_RUN=0 to release)
