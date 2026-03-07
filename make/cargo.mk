# Scope: canonical Rust cargo gates delegated to cargo-native tooling.
# Public targets: audit check coverage fmt lint test test-slow test-all
SHELL := /bin/bash

CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always

ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
CARGO_TARGET_DIR ?= $(CURDIR)/artifacts/target
NEXTEST_CACHE_DIR ?= $(CURDIR)/artifacts/target/nextest

cleanup_root_nextest = \
	if [ -d "$(CURDIR)/target/nextest" ]; then rm -rf "$(CURDIR)/target/nextest"; fi; \
	if [ -d "$(CURDIR)/target" ]; then rm -rf "$(CURDIR)/target"; fi

nextest_summary = \
	summary_line=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | grep 'Summary \[' | tail -n 1); \
	set -- $$(printf '%s\n' "$$summary_line" | awk ' \
		{ \
			for (i = 1; i <= NF; i++) { \
				prev = (i > 1) ? $$(i - 1) : $$1; \
				gsub(/[^0-9]/, "", prev); \
				if ($$i ~ /^test/) total = prev; \
				else if ($$i ~ /^passed/) passed = prev; \
				else if ($$i ~ /^failed/) failed = prev; \
				else if ($$i ~ /^skipped/) skipped = prev; \
			} \
		} \
		END { \
			printf "%s %s %s %s\n", total + 0, passed + 0, failed + 0, skipped + 0; \
		}'); \
	total=$$1; \
	passed=$$2; \
	failed=$$3; \
	skipped=$$4; \
	leaky=$$(grep -c ' LEAK ' "$$report_file" || true); \
	max_list_items=50; \
	failed_tests=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | awk '/ FAIL / { test_name = $$0; sub(/^.* FAIL \[[^]]*\] \([^)]*\) /, "", test_name); seen[test_name] = 1 } END { for (test_name in seen) print test_name }' | LC_ALL=C sort); \
	skipped_tests=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | awk '/ SKIP / { test_name = $$0; sub(/^.* SKIP \[[^]]*\] \([^)]*\) /, "", test_name); seen[test_name] = 1 } END { for (test_name in seen) print test_name }' | LC_ALL=C sort); \
	leaky_tests=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file" | awk '/ LEAK / { test_name = $$0; sub(/^.* LEAK \[[^]]*\] \([^)]*\) /, "", test_name); seen[test_name] = 1 } END { for (test_name in seen) print test_name }' | LC_ALL=C sort); \
	print_test_group() { \
		label="$$1"; color="$$2"; tests="$$3"; \
		[ -n "$$tests" ] || return 0; \
		total_items=$$(printf '%s\n' "$$tests" | sed '/^$$/d' | wc -l | tr -d ' '); \
		printf '\033[%sm%s\033[0m\n' "$$color" "$$label"; \
		printf '%s\n' "$$tests" | sed '/^$$/d' | head -n "$$max_list_items" | sed 's/^/  /'; \
		if [ "$$total_items" -gt "$$max_list_items" ]; then \
			printf '  ... %s more\n' "$$((total_items - max_list_items))"; \
		fi; \
	}; \
	printf '\033[1;36m%s\033[0m total=%s \033[1;32mpassed=%s\033[0m \033[1;31mfailed=%s\033[0m \033[1;33mskipped=%s\033[0m \033[1;35mleaky=%s\033[0m\n' "nextest-summary:" "$$total" "$$passed" "$$failed" "$$skipped" "$$leaky"; \
	print_test_group "failed-tests:" "1;31" "$$failed_tests"; \
	print_test_group "leaky-tests:" "1;35" "$$leaky_tests"; \
	print_test_group "skipped-tests:" "1;33" "$$skipped_tests"

audit: ## Run cargo dependency audit
	@$(call require_tool,cargo-audit); CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo audit

check: ## Run cargo check for the workspace
	@CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo check --workspace --all-targets

coverage: ## Run workspace coverage with cargo llvm-cov + nextest
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@mkdir -p artifacts/coverage
	@mkdir -p artifacts/coverage/profraw
	@status=0; \
	LLVM_PROFILE_FILE="$(CURDIR)/artifacts/coverage/profraw/default_%m_%p.profraw" CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo llvm-cov nextest --color always --workspace --all-features --lcov --output-path artifacts/coverage/lcov.info --config-file configs/nextest/nextest.toml --user-config-file none --run-ignored all --cargo-quiet || status=$$?; \
	$(cleanup_root_nextest); \
	test $$status -eq 0
	@CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo llvm-cov report

fmt: ## Run cargo fmt --check
	@printf '%s\n' "run: cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml"
	@mkdir -p $(ARTIFACT_ROOT)/fmt/$(RUN_ID)
	@output="$$(CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo fmt --all -- --check --config-path configs/rust/rustfmt.toml 2>&1)"; \
	status=$$?; \
	printf '%s\n' "$$output" | tee $(ARTIFACT_ROOT)/fmt/$(RUN_ID)/report.txt; \
	if [ $$status -eq 0 ]; then \
		printf '%s\n' "fmt check complete"; \
	fi; \
	exit $$status

lint: ## Run cargo clippy with warnings denied
	@printf '%s\n' "run: cargo clippy --workspace --all-targets --all-features --locked -- -D warnings"
	@mkdir -p $(ARTIFACT_ROOT)/lint/$(RUN_ID)
	@CLIPPY_CONF_DIR=configs/rust CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) cargo clippy --workspace --all-targets --all-features --locked -- -D warnings 2>&1 | tee $(ARTIFACT_ROOT)/lint/$(RUN_ID)/report.txt

test: ## Run workspace tests with cargo nextest
	@$(call require_tool,cargo-nextest)
	@printf '%s\n' "run: cargo nextest run --workspace --profile $${NEXTEST_PROFILE:-default} --status-level $${NEXTEST_STATUS_LEVEL:-all} --final-status-level $${NEXTEST_FINAL_STATUS_LEVEL:-all} --show-progress $${NEXTEST_SHOW_PROGRESS:-counter}"
	@mkdir -p $(ARTIFACT_ROOT)/test/$(RUN_ID)
	@status=0; report_file="$(ARTIFACT_ROOT)/test/$(RUN_ID)/nextest.log"; \
	cleanup() { $(cleanup_root_nextest); }; trap cleanup EXIT INT TERM; \
	CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo nextest run --color always --workspace --config-file configs/nextest/nextest.toml --user-config-file none --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" --status-level "$${NEXTEST_STATUS_LEVEL:-all}" --final-status-level "$${NEXTEST_FINAL_STATUS_LEVEL:-all}" --show-progress "$${NEXTEST_SHOW_PROGRESS:-counter}" -E "$${NEXTEST_FILTER_EXPR:-not test(/(^|::)slow_/)}" 2>&1 | tee "$$report_file"; \
	status=$${PIPESTATUS:-$${pipestatus}}; \
	$(nextest_summary); \
	trap - EXIT INT TERM; cleanup; \
	test $$status -eq 0

test-slow: ## Run only slow_ tests with cargo nextest
	@$(call require_tool,cargo-nextest)
	@printf '%s\n' "run: cargo nextest run --workspace --profile $${NEXTEST_PROFILE:-default} --status-level $${NEXTEST_STATUS_LEVEL:-all} --final-status-level $${NEXTEST_FINAL_STATUS_LEVEL:-all} --show-progress $${NEXTEST_SHOW_PROGRESS:-counter} -E test(/(^|::)slow_/)"
	@mkdir -p $(ARTIFACT_ROOT)/test/$(RUN_ID)
	@status=0; report_file="$(ARTIFACT_ROOT)/test/$(RUN_ID)/nextest-slow.log"; \
	cleanup() { $(cleanup_root_nextest); }; trap cleanup EXIT INT TERM; \
	CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo nextest run --color always --cargo-quiet --workspace --config-file configs/nextest/nextest.toml --user-config-file none --target-dir "$(CARGO_TARGET_DIR)" --profile "$${NEXTEST_PROFILE:-default}" --status-level "$${NEXTEST_STATUS_LEVEL:-all}" --final-status-level "$${NEXTEST_FINAL_STATUS_LEVEL:-all}" --show-progress "$${NEXTEST_SHOW_PROGRESS:-counter}" -E "test(/(^|::)slow_/)" 2>&1 | tee "$$report_file"; \
	status=$${PIPESTATUS:-$${pipestatus}}; \
	$(nextest_summary); \
	trap - EXIT INT TERM; cleanup; \
	test $$status -eq 0

test-all: ## Run all workspace tests including slow_ and ignored tests
	@$(call require_tool,cargo-nextest)
	@printf '%s\n' "run: cargo nextest run --workspace --all-features --run-ignored all --retries 0 --profile $${NEXTEST_PROFILE:-default} --status-level $${NEXTEST_STATUS_LEVEL:-all} --final-status-level $${NEXTEST_FINAL_STATUS_LEVEL:-all} --show-progress $${NEXTEST_SHOW_PROGRESS:-counter}"
	@mkdir -p $(ARTIFACT_ROOT)/test/$(RUN_ID)
	@status=0; report_file="$(ARTIFACT_ROOT)/test/$(RUN_ID)/nextest-all.log"; \
	cleanup() { $(cleanup_root_nextest); }; trap cleanup EXIT INT TERM; \
	CARGO_TERM_COLOR=$(CARGO_TERM_COLOR) CARGO_TERM_PROGRESS_WHEN=$(CARGO_TERM_PROGRESS_WHEN) CARGO_TERM_PROGRESS_WIDTH=$(CARGO_TERM_PROGRESS_WIDTH) CARGO_TERM_VERBOSE=$(CARGO_TERM_VERBOSE) NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" cargo nextest run --color always --cargo-quiet --workspace --all-features --config-file configs/nextest/nextest.toml --user-config-file none --target-dir "$(CARGO_TARGET_DIR)" --run-ignored all --retries 0 --profile "$${NEXTEST_PROFILE:-default}" --status-level "$${NEXTEST_STATUS_LEVEL:-all}" --final-status-level "$${NEXTEST_FINAL_STATUS_LEVEL:-all}" --show-progress "$${NEXTEST_SHOW_PROGRESS:-counter}" 2>&1 | tee "$$report_file"; \
	status=$${PIPESTATUS:-$${pipestatus}}; \
	$(nextest_summary); \
	trap - EXIT INT TERM; cleanup; \
	test $$status -eq 0

.PHONY: audit check coverage fmt lint test test-slow test-all
