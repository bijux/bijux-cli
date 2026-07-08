# Rust quality checks and reports that write only under `artifacts/`.

RS_ARTIFACT_ROOT ?= artifacts/rust
RS_RUN_ID ?= local

RS_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/target)
RS_NEXTEST_CACHE_DIR ?= $(RS_TARGET_DIR)/nextest
RS_NEXTEST_CONFIG_HOME ?= $(abspath $(RS_ARTIFACT_ROOT)/nextest/config)
RS_PROFRAW_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/profraw)
RS_LLVM_PROFILE_FILE ?= $(RS_PROFRAW_DIR)/default_%m_%p.profraw
RS_COVERAGE_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/target)

RS_FMT_REPORT ?= $(RS_ARTIFACT_ROOT)/fmt/$(RS_RUN_ID)/report.txt
RS_LINT_REPORT ?= $(RS_ARTIFACT_ROOT)/lint/$(RS_RUN_ID)/report.txt
RS_TEST_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest.log
RS_TEST_ALL_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-all.log
RS_AUDIT_REPORT ?= $(RS_ARTIFACT_ROOT)/audit/$(RS_RUN_ID)/report.txt
RS_COVERAGE_DIR ?= $(RS_ARTIFACT_ROOT)/coverage/$(RS_RUN_ID)
RS_LCOV_FILE ?= $(RS_COVERAGE_DIR)/lcov.info
RS_COVERAGE_TEST_REPORT ?= $(RS_COVERAGE_DIR)/nextest.log
RS_COVERAGE_SUMMARY_REPORT ?= $(RS_COVERAGE_DIR)/summary.txt
RS_RELEASE_VALIDATION_DIR ?= $(RS_ARTIFACT_ROOT)/release-validation/$(RS_RUN_ID)
RS_RELEASE_TREE_DIR ?= $(abspath $(RS_RELEASE_VALIDATION_DIR)/workspace)
RS_RELEASE_CARGO_CONFIG ?= $(RS_RELEASE_TREE_DIR)/.cargo/config.toml
RS_RELEASE_VALIDATION_TARGET_DIR ?= $(abspath $(RS_RELEASE_VALIDATION_DIR)/target)
RS_RELEASE_TREE_VERSION_FILE ?= $(RS_RELEASE_VALIDATION_DIR)/workspace-version.txt
RS_RELEASE_FMT_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/fmt.txt
RS_RELEASE_CLIPPY_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/clippy.txt
RS_RELEASE_TEST_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/test.txt
RS_RELEASE_DOC_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/doc.txt
RS_RELEASE_PACKAGE_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/package.txt
RS_RELEASE_PUBLISH_DRY_RUN_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/publish-dry-run.txt
RS_RELEASE_SMOKE_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/smoke.txt
RS_DEV_CLI_BIN ?= $(RS_TARGET_DIR)/debug/bijux-dev-cli
RS_DAG_BIN ?= $(RS_TARGET_DIR)/debug/bijux-dag
RS_RELEASE_DAG_BIN ?= $(RS_RELEASE_VALIDATION_TARGET_DIR)/debug/bijux-dag
RS_RELEASE_BUNDLE_DIR ?= $(RS_ARTIFACT_ROOT)/build
DAG_RELEASE_PACKAGE ?= bijux-dag-cli
DAG_RELEASE_BIN ?= bijux-dag
RS_BUILD_GIT_SHA ?= $(shell git rev-parse --short HEAD 2>/dev/null || true)
RS_BUILD_GIT_SHA_ENV ?= $(if $(strip $(RS_BUILD_GIT_SHA)),BIJUX_DAG_BUILD_GIT_SHA="$(strip $(RS_BUILD_GIT_SHA))")
RUST_PUBLIC_DAG_PACKAGES ?= bijux-dag-core bijux-dag-artifacts bijux-dag-runtime bijux-dag-app bijux-dag-cli
RUST_PUBLISH_PACKAGES ?= bijux-dag-core bijux-dag-artifacts bijux-dag-runtime bijux-dag-app bijux-dag-cli bijux-cli
RUST_PUBLISH_DRY_RUN ?= 1
RUST_PUBLISH_SKIP_EXISTING ?= 1
RUST_PUBLISH_ALLOW_DIRTY ?= 0
RUST_PUBLISH_REGISTRY ?= crates-io

CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always

NEXTEST_PROFILE ?= default
NEXTEST_RELEASE_PROFILE ?= ci
NEXTEST_FULL_PROFILE ?= ci
NEXTEST_STATUS_LEVEL ?= all
NEXTEST_FINAL_STATUS_LEVEL ?= all
# Default fast-lane exclusions for tests that consistently exceed 10 seconds.
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

.PHONY: fmt-rs lint-rs test-rs test-release-rs test-all-rs prepare-release-tree-rs fmt-release-rs clippy-release-rs test-release-workspace-rs doc-release-rs package-release-rs publish-dry-run-release-rs smoke-release-rs release-validate-rs coverage coverage-rs audit-rs publish-rs build-dag-release-bundle
.NOTPARALLEL: prepare-release-tree-rs fmt-release-rs clippy-release-rs test-release-workspace-rs doc-release-rs package-release-rs publish-dry-run-release-rs smoke-release-rs release-validate-rs

##@ Rust
fmt-rs: ## Run Rust formatting checks
	@mkdir -p "$(dir $(RS_FMT_REPORT))"
	@printf '%s\n' "run: cargo fmt --all -- --check"
	@set -o pipefail; \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo fmt --all -- --check 2>&1 | tee "$(RS_FMT_REPORT)"

lint-rs: ## Run Rust clippy checks with -D warnings
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

test-rs: ## Run the Rust fast suite and skip known tests over 10 seconds
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(dir $(RS_TEST_REPORT))" "$(RS_PROFRAW_DIR)" "$(RS_NEXTEST_CONFIG_HOME)"
	@printf '%s\n' "prepare: cargo build -p bijux-dev --bin bijux-dev-cli && cargo build -p bijux-dag-cli --bin bijux-dag"
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build -p bijux-dev --bin bijux-dev-cli
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build -p bijux-dag-cli --bin bijux-dag
	@status=0; \
	filter_expr="$${NEXTEST_FILTER_EXPR:-$(NEXTEST_SLOW_EXCLUDE_EXPR)}"; \
	BIJUX_DEV_CLI_BIN="$(RS_DEV_CLI_BIN)" \
	BIJUX_DAG_BIN="$(RS_DAG_BIN)" \
	LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	XDG_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo nextest run \
		--workspace \
		--config-file configs/rust/nextest.toml \
		--profile "$(NEXTEST_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		$${filter_expr:+-E "$${filter_expr}"} \
		2>&1 | tee "$(RS_TEST_REPORT)" || status=$$?; \
	$(call rs_nextest_summary,$(RS_TEST_REPORT)); \
	test $$status -eq 0

test-release-rs: ## Run the required Rust release-candidate lane
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(dir $(RS_TEST_REPORT))" "$(RS_PROFRAW_DIR)" "$(RS_NEXTEST_CONFIG_HOME)"
	@printf '%s\n' "prepare: cargo build -p bijux-dev --bin bijux-dev-cli && cargo build -p bijux-dag-cli --bin bijux-dag"
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build -p bijux-dev --bin bijux-dev-cli
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build -p bijux-dag-cli --bin bijux-dag
	@status=0; \
	filter_expr="$${NEXTEST_FILTER_EXPR:-$(NEXTEST_SLOW_EXCLUDE_EXPR)}"; \
	BIJUX_DEV_CLI_BIN="$(RS_DEV_CLI_BIN)" \
	BIJUX_DAG_BIN="$(RS_DAG_BIN)" \
	LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	XDG_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" \
	CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
	NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo nextest run \
		--workspace \
		--config-file configs/rust/nextest.toml \
		--profile "$(NEXTEST_RELEASE_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		$${filter_expr:+-E "$${filter_expr}"} \
		2>&1 | tee "$(RS_TEST_REPORT)" || status=$$?; \
	$(call rs_nextest_summary,$(RS_TEST_REPORT)); \
	test $$status -eq 0

test-all-rs: ## Run the full Rust suite, including ignored tests
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(dir $(RS_TEST_ALL_REPORT))" "$(RS_PROFRAW_DIR)" "$(RS_NEXTEST_CONFIG_HOME)"
	@printf '%s\n' "prepare: cargo build -p bijux-dev --bin bijux-dev-cli && cargo build -p bijux-dag-cli --bin bijux-dag"
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build -p bijux-dev --bin bijux-dev-cli
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build -p bijux-dag-cli --bin bijux-dag
	@status=0; \
	BIJUX_DEV_CLI_BIN="$(RS_DEV_CLI_BIN)" \
	BIJUX_DAG_BIN="$(RS_DAG_BIN)" \
	LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	XDG_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" \
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
		--profile "$(NEXTEST_FULL_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		$${NEXTEST_FILTER_EXPR:+-E "$${NEXTEST_FILTER_EXPR}"} \
		2>&1 | tee "$(RS_TEST_ALL_REPORT)" || status=$$?; \
	$(call rs_nextest_summary,$(RS_TEST_ALL_REPORT)); \
	test $$status -eq 0

prepare-release-tree-rs: ## Prepare a clean release-candidate tree from committed HEAD
	@mkdir -p "$(RS_RELEASE_VALIDATION_DIR)"
	@rm -rf "$(RS_RELEASE_TREE_DIR)"
	@release_version="$$(python3 -c "import tomllib; from pathlib import Path; print(tomllib.load(Path('Cargo.toml').open('rb'))['workspace']['package']['version'])")"; \
	printf '%s\n' "$$release_version" > "$(RS_RELEASE_TREE_VERSION_FILE)"; \
	printf '%s\n' "prepare: release tree version $$release_version"; \
	python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$(RS_RELEASE_TREE_DIR)" --version "$$release_version" >/dev/null
	@mkdir -p "$(dir $(RS_RELEASE_CARGO_CONFIG))"
	# Patch staged public DAG crates into the clean release tree so dry-run publish verifies topological public dependencies before the new release is present on crates.io.
	@printf '\n[patch.crates-io]\nbijux-dag-core = { path = "crates/bijux-dag-core" }\nbijux-dag-artifacts = { path = "crates/bijux-dag-artifacts" }\nbijux-dag-runtime = { path = "crates/bijux-dag-runtime" }\nbijux-dag-app = { path = "crates/bijux-dag-app" }\nbijux-dag-cli = { path = "crates/bijux-dag-cli" }\n' >> "$(RS_RELEASE_CARGO_CONFIG)"

fmt-release-rs: prepare-release-tree-rs ## Run release-candidate formatting validation in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_FMT_REPORT))"
	@printf '%s\n' "run: cargo fmt --all -- --check"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo fmt --all -- --check 2>&1 | tee "$(abspath $(RS_RELEASE_FMT_REPORT))"

clippy-release-rs: prepare-release-tree-rs ## Run release-candidate clippy validation in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_CLIPPY_REPORT))"
	@printf '%s\n' "run: cargo clippy --workspace --all-targets --all-features --locked -- -D warnings"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	CLIPPY_CONF_DIR="configs/rust" \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings 2>&1 | tee "$(abspath $(RS_RELEASE_CLIPPY_REPORT))"

test-release-workspace-rs: prepare-release-tree-rs ## Run release-candidate workspace tests in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_TEST_REPORT))"
	@printf '%s\n' "run: cargo test --workspace --all-targets --all-features --locked"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" cargo build -p bijux-dag-cli --bin bijux-dag; \
	$(RS_BUILD_GIT_SHA_ENV) \
	BIJUX_DAG_BIN="$(RS_RELEASE_DAG_BIN)" \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo test --workspace --all-targets --all-features --locked 2>&1 | tee "$(abspath $(RS_RELEASE_TEST_REPORT))"

doc-release-rs: prepare-release-tree-rs ## Run release-candidate docs build in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_DOC_REPORT))"
	@printf '%s\n' "run: cargo doc --workspace --all-features --no-deps"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo doc --workspace --all-features --no-deps 2>&1 | tee "$(abspath $(RS_RELEASE_DOC_REPORT))"

package-release-rs: prepare-release-tree-rs ## Run release-candidate cargo package listings for public DAG crates
	@mkdir -p "$(dir $(RS_RELEASE_PACKAGE_REPORT))"
	@rm -f "$(RS_RELEASE_PACKAGE_REPORT)"
	@set -euo pipefail; \
	for package in $(RUST_PUBLIC_DAG_PACKAGES); do \
		printf '%s\n' "run: cargo package -p $${package} --list" | tee -a "$(RS_RELEASE_PACKAGE_REPORT)"; \
		( \
			cd "$(RS_RELEASE_TREE_DIR)"; \
			$(RS_BUILD_GIT_SHA_ENV) \
			CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
			CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
			CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
			CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
			CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
			cargo package -p "$${package}" --list \
		) 2>&1 | tee -a "$(RS_RELEASE_PACKAGE_REPORT)"; \
	done

publish-dry-run-release-rs: prepare-release-tree-rs ## Run release-candidate cargo publish dry-runs for public DAG crates
	@mkdir -p "$(dir $(RS_RELEASE_PUBLISH_DRY_RUN_REPORT))"
	@rm -f "$(RS_RELEASE_PUBLISH_DRY_RUN_REPORT)"
	@set -euo pipefail; \
	for package in $(RUST_PUBLIC_DAG_PACKAGES); do \
		printf '%s\n' "run: cargo publish -p $${package} --dry-run --locked" | tee -a "$(RS_RELEASE_PUBLISH_DRY_RUN_REPORT)"; \
		( \
			cd "$(RS_RELEASE_TREE_DIR)"; \
			$(RS_BUILD_GIT_SHA_ENV) \
			CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
			CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
			CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
			CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
			CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
			cargo publish -p "$${package}" --dry-run --locked \
		) 2>&1 | tee -a "$(RS_RELEASE_PUBLISH_DRY_RUN_REPORT)"; \
	done

smoke-release-rs: prepare-release-tree-rs ## Run release-candidate DAG CLI smoke tests in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_SMOKE_REPORT))"
	@printf '%s\n' "run: cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture 2>&1 | tee "$(abspath $(RS_RELEASE_SMOKE_REPORT))"

release-validate-rs: fmt-release-rs clippy-release-rs test-release-workspace-rs doc-release-rs package-release-rs publish-dry-run-release-rs smoke-release-rs ## Run the canonical Rust release validation suite

coverage: coverage-rs ## Run coverage and refresh tracked coverage reports
	@mkdir -p artifacts/coverage
	@cp "$(RS_LCOV_FILE)" artifacts/coverage/lcov.info
	@BIJUX_COVERAGE_LCOV_PATH="$(RS_LCOV_FILE)" cargo run --locked -p bijux-dev --bin generate_line_coverage_reports

coverage-rs: ## Run Rust coverage with llvm-cov and emit reports
	$(call rs_require_tool,cargo-llvm-cov)
	$(call rs_require_tool,cargo-nextest)
	@mkdir -p "$(RS_COVERAGE_DIR)" "$(RS_PROFRAW_DIR)" "$(RS_NEXTEST_CONFIG_HOME)"
	@printf '%s\n' "prepare: cargo build -p bijux-dev --bin bijux-dev-cli && cargo build -p bijux-dag-cli --bin bijux-dag"
	@CARGO_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" cargo build -p bijux-dev --bin bijux-dev-cli
	@CARGO_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" cargo build -p bijux-dag-cli --bin bijux-dag
	@status=0; \
	BIJUX_DEV_CLI_BIN="$(RS_COVERAGE_TARGET_DIR)/debug/bijux-dev-cli" \
	BIJUX_DAG_BIN="$(RS_COVERAGE_TARGET_DIR)/debug/bijux-dag" \
	LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" \
	XDG_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" \
	CARGO_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" \
	CARGO_LLVM_COV_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" \
	NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo llvm-cov nextest \
		--workspace \
		--run-ignored all \
		--retries 0 \
		--config-file configs/rust/nextest.toml \
		--profile "$(NEXTEST_PROFILE)" \
		--status-level "$(NEXTEST_STATUS_LEVEL)" \
		--final-status-level "$(NEXTEST_FINAL_STATUS_LEVEL)" \
		$${NEXTEST_FILTER_EXPR:+-E "$${NEXTEST_FILTER_EXPR}"} \
		2>&1 | tee "$(RS_COVERAGE_TEST_REPORT)" || status=$$?; \
	printf '%s' "$$status" > "$(RS_COVERAGE_DIR)/status.code"; \
	$(call rs_nextest_summary,$(RS_COVERAGE_TEST_REPORT)); \
	true
	@set -o pipefail; \
	CARGO_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" \
	CARGO_LLVM_COV_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" \
	cargo llvm-cov report --summary-only 2>&1 | tee "$(RS_COVERAGE_SUMMARY_REPORT)"
	@set -o pipefail; \
	CARGO_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" \
	CARGO_LLVM_COV_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" \
	cargo llvm-cov report --lcov --output-path "$(RS_LCOV_FILE)" >/dev/null
	@total_line=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$(RS_COVERAGE_SUMMARY_REPORT)" | grep '^TOTAL' | tail -n 1 || true); \
	printf '\033[1;36m%s\033[0m %s\n' "coverage-summary:" "$${total_line:-unavailable}"; \
	printf '\033[1;36m%s\033[0m %s\n' "coverage-lcov:" "$(RS_LCOV_FILE)"; \
	printf '\033[1;36m%s\033[0m %s\n' "coverage-report:" "$(RS_COVERAGE_SUMMARY_REPORT)"
	@test "$$(cat "$(RS_COVERAGE_DIR)/status.code")" -eq 0

audit-rs: ## Run cargo-deny and cargo-audit
	$(call rs_require_tool,cargo-deny)
	$(call rs_require_tool,cargo-audit)
	@mkdir -p "$(dir $(RS_AUDIT_REPORT))"
	@set -o pipefail; \
	deny_status=0; \
	audit_status=0; \
	{ \
		echo "run: cargo deny check bans licenses sources --config configs/rust/deny.toml"; \
		CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo deny check bans licenses sources --config configs/rust/deny.toml || deny_status=$$?; \
		echo; \
		echo "run: cargo run -q -p bijux-dev --bin bijux-dev-dag -- security"; \
		CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo run -q -p bijux-dev --bin bijux-dev-dag -- security || audit_status=$$?; \
	} 2>&1 | tee "$(RS_AUDIT_REPORT)"; \
	test $$deny_status -eq 0; \
	test $$audit_status -eq 0

publish-rs: ## Publish Rust crates and dry-run by default
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
	workspace_root="."; \
	temp_root=""; \
	if [ -n "$(RELEASE_VERSION)" ]; then \
		temp_root="$$(mktemp -d "$${TMPDIR:-/tmp}/bijux-release-tree.XXXXXX")"; \
		trap 'test -n "$${temp_root}" && rm -rf "$${temp_root}"' EXIT; \
		python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$${temp_root}" --version "$(RELEASE_VERSION)" >/dev/null; \
		workspace_root="$${temp_root}"; \
		echo "→ Publishing from release tree stamped to $(RELEASE_VERSION)"; \
	elif [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ]; then \
		workspace_version="$$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p['\''name'\'']: p['\''version'\''] for p in data['\''packages'\'']}; print(pkgs.get('\''bijux-cli'\'', '\'''\''))' 2>/dev/null)"; \
		case "$${workspace_version}" in \
			*-*) if [ "$(PUBLISH_ALLOW_PRERELEASE)" != "1" ]; then \
				echo "Refusing to publish prerelease workspace version $${workspace_version} without RELEASE_VERSION or PUBLISH_ALLOW_PRERELEASE=1"; \
				exit 1; \
			fi ;; \
		esac; \
	fi; \
	for pkg in $(RUST_PUBLISH_PACKAGES); do \
		publish_version="$$(cargo metadata --manifest-path "$${workspace_root}/Cargo.toml" --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p["name"]: p["version"] for p in data["packages"]}; print(pkgs.get(sys.argv[1], ""))' "$$pkg" 2>/dev/null)"; \
		if [ -z "$${publish_version}" ]; then \
			echo "Could not resolve version for package $$pkg from cargo metadata"; \
			exit 1; \
		fi; \
		if [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ] && [ "$(RUST_PUBLISH_SKIP_EXISTING)" = "1" ]; then \
			status="$$(curl -s -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$$pkg/$${publish_version}" || true)"; \
			if [ "$${status}" = "200" ]; then \
				echo "→ Skipping $$pkg $${publish_version}; already present on crates.io"; \
				continue; \
			fi; \
		fi; \
		echo "→ cargo publish -p $$pkg@$${publish_version} --registry $(RUST_PUBLISH_REGISTRY) $$dry_run_flag"; \
		publish_log="$$(mktemp "$${TMPDIR:-/tmp}/bijux-cargo-publish.XXXXXX.log")"; \
		if $(RS_BUILD_GIT_SHA_ENV) CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
			cargo publish \
				--locked \
				--manifest-path "$${workspace_root}/Cargo.toml" \
				--registry "$(RUST_PUBLISH_REGISTRY)" \
				-p "$$pkg" \
				$$allow_dirty_flag \
				$$dry_run_flag >"$${publish_log}" 2>&1; then \
			cat "$${publish_log}"; \
			rm -f "$${publish_log}"; \
		else \
			cat "$${publish_log}"; \
			if [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ] && [ "$(RUST_PUBLISH_SKIP_EXISTING)" = "1" ] && \
				grep -Eq 'already exists on crates\.io index|already uploaded' "$${publish_log}"; then \
				echo "→ Skipping $$pkg $${publish_version}; registry already has this release"; \
				rm -f "$${publish_log}"; \
				continue; \
			fi; \
			rm -f "$${publish_log}"; \
			exit 1; \
		fi; \
	done

build-dag-release-bundle: ## Build a stamped bijux-dag binary release bundle under artifacts/rust/build
	@mkdir -p "$(RS_RELEASE_BUNDLE_DIR)"
	@set -euo pipefail; \
	workspace_root="."; \
	temp_root=""; \
	if [ -n "$(RELEASE_VERSION)" ]; then \
		temp_root="$$(mktemp -d "$${TMPDIR:-/tmp}/bijux-release-tree.XXXXXX")"; \
		trap 'test -n "$${temp_root}" && rm -rf "$${temp_root}"' EXIT; \
		python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$${temp_root}" --version "$(RELEASE_VERSION)" >/dev/null; \
		workspace_root="$${temp_root}"; \
		echo "→ Building DAG release bundle from release tree stamped to $(RELEASE_VERSION)"; \
	fi; \
	host_triple="$$(rustc -vV | awk '/^host:/ {print $$2}')"; \
	bundle_version="$$(cargo metadata --manifest-path "$${workspace_root}/Cargo.toml" --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p["name"]: p["version"] for p in data["packages"]}; print(pkgs.get(sys.argv[1], ""))' "$(DAG_RELEASE_PACKAGE)" 2>/dev/null)"; \
	if [ -z "$${bundle_version}" ]; then \
		echo "Could not resolve version for package $(DAG_RELEASE_PACKAGE) from cargo metadata"; \
		exit 1; \
	fi; \
	stage_dir="$(RS_RELEASE_BUNDLE_DIR)/$(DAG_RELEASE_BIN)-bundle"; \
	archive_name="$(DAG_RELEASE_BIN)-v$${bundle_version}-$${host_triple}.tar.gz"; \
	archive_path="$(RS_RELEASE_BUNDLE_DIR)/$${archive_name}"; \
	rm -rf "$${stage_dir}"; \
	rm -f "$${archive_path}" "$${archive_path}.sha256"; \
	mkdir -p "$${stage_dir}/bin"; \
	$(RS_BUILD_GIT_SHA_ENV) CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build --release --locked --manifest-path "$${workspace_root}/Cargo.toml" -p "$(DAG_RELEASE_PACKAGE)" --bin "$(DAG_RELEASE_BIN)"; \
	cp "$(RS_TARGET_DIR)/release/$(DAG_RELEASE_BIN)" "$${stage_dir}/bin/$(DAG_RELEASE_BIN)"; \
	cp "$${workspace_root}/LICENSE" "$${stage_dir}/LICENSE"; \
	cp "$${workspace_root}/crates/$(DAG_RELEASE_PACKAGE)/README.md" "$${stage_dir}/README.md"; \
	printf 'version=%s\ncrate=%s\nbinary=%s\nhost_triple=%s\n' "$${bundle_version}" "$(DAG_RELEASE_PACKAGE)" "$(DAG_RELEASE_BIN)" "$${host_triple}" > "$${stage_dir}/release-metadata.txt"; \
	printf '%s\n' \
		'Install by extracting this archive and placing `bin/$(DAG_RELEASE_BIN)` on your PATH.' \
		'For source publication and API documentation, use the published `$(DAG_RELEASE_PACKAGE)` crate.' \
		> "$${stage_dir}/INSTALL.txt"; \
	( \
		cd "$${stage_dir}"; \
		find . -type f ! -name 'checksums.txt' -print | LC_ALL=C sort | while IFS= read -r file_path; do \
			shasum -a 256 "$${file_path}"; \
		done > checksums.txt; \
	); \
	tar -C "$${stage_dir}" -czf "$${archive_path}" .; \
	shasum -a 256 "$${archive_path}" > "$${archive_path}.sha256"; \
	echo "→ Built DAG release bundle: $${archive_path}"
