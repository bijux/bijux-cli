SHELL := /bin/bash

ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
CARGO_TARGET_DIR ?= $(CURDIR)/artifacts/target
NEXTEST_CACHE_DIR ?= $(CURDIR)/artifacts/target/nextest
LLVM_PROFILE_FILE ?= $(CURDIR)/artifacts/coverage/profraw/default_%m_%p.profraw

DEV_TOOL := LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" RUSTFLAGS="-Aunused-crate-dependencies" cargo run -p bijux-dev-dag --bin bijux-dev-dag --
RUN_DIR := artifacts/runs
REPORT_DIR := artifacts/reports
TARGET_DIR := artifacts/target
CONTRACT_DIR := artifacts/contracts
OUTPUT_PATHS := $(RUN_DIR) $(REPORT_DIR) $(TARGET_DIR) $(CONTRACT_DIR)

include make/macros.mk
include make/cargo.mk
include make/evidence.mk

.DEFAULT_GOAL := help

define run_or_fail
	@echo "--> $(1)"
	@$(2) || (\
		echo "--> failed: $(1)" >&2; \
		echo "--> inspect artifacts: $(OUTPUT_PATHS)" >&2; \
		echo "--> pass --report <path> to write machine-readable output" >&2; \
		exit 1)
endef

all: test ## Run default test gate

checks: ## Run default control-plane checks
	$(call run_or_fail,Run default checks,$(DEV_TOOL) checks run --domain policy --domain quality --domain style --domain supply-chain)

checks-fast: ## Run fast control-plane checks
	$(call run_or_fail,Run fast checks,$(DEV_TOOL) checks run)

checks-all: ## Run full control-plane checks
	@mkdir -p $(ARTIFACT_ROOT)/checks/$(RUN_ID)
	@status=0; report_file="$(ARTIFACT_ROOT)/checks/$(RUN_ID)/checks-all.log"; \
	printf '%s\n' "run: $(DEV_TOOL) checks run" | tee "$$report_file"; \
	$(DEV_TOOL) checks run 2>&1 | tee -a "$$report_file"; \
	status=$${PIPESTATUS:-$${pipestatus}}; \
	clean_report=$$(perl -pe 's/\e\[[0-9;]*[[:alpha:]]//g' "$$report_file"); \
	summary_total=$$(printf '%s\n' "$$clean_report" | grep -E -c '^\[checks\.[^]]+\] ' || true); \
	summary_passed=$$(printf '%s\n' "$$clean_report" | grep -E -c '^\[checks\.[^]]+\] ok ' || true); \
	summary_failed=$$(printf '%s\n' "$$clean_report" | grep -E -c '^\[checks\.[^]]+\] error ' || true); \
	summary_skipped=$$((summary_total - summary_passed - summary_failed)); \
	summary_failed_items=$$(printf '%s\n' "$$clean_report" | awk '/^\[checks\.[^]]+\] error / { name=$$1; gsub(/^\[/, "", name); gsub(/\]$$/, "", name); print name }'); \
	if [ "$$summary_total" -eq 0 ]; then \
		summary_total=1; \
		if [ "$$status" -eq 0 ]; then summary_passed=1; summary_failed=0; else summary_passed=0; summary_failed=1; summary_failed_items="checks-all"; fi; \
		summary_skipped=0; \
	fi; \
	printf '\033[1;36m%s\033[0m total=%s \033[1;32mpassed=%s\033[0m \033[1;31mfailed=%s\033[0m \033[1;33mskipped=%s\033[0m \033[1;35mleaky=%s\033[0m\n' "nextest-summary:" "$$summary_total" "$$summary_passed" "$$summary_failed" "$$summary_skipped" "0"; \
	if [ "$$summary_failed" -gt 0 ]; then \
		printf '\033[1;31m%s\033[0m\n' "failed-tests:"; \
		printf '%s\n' "$$summary_failed_items" | sed '/^$$/d' | sed 's/^/  /'; \
	fi; \
	test $$status -eq 0

docs: ## Run documentation checks
	$(call run_or_fail,Run documentation checks,$(DEV_TOOL) docs run)

security: ## Run security checks
	$(call run_or_fail,Run security checks,$(DEV_TOOL) checks run --domain supply-chain)

compat: ## Run compatibility contracts
	$(call run_or_fail,Run compatibility contracts,$(DEV_TOOL) contracts run --domain compat)

golden: ## Run runtime contract checks
	$(call run_or_fail,Run runtime golden contract,$(DEV_TOOL) contracts run --domain runtime)

tests-all: ## Run all control-plane test suites
	$(call run_or_fail,Run all test suites,$(DEV_TOOL) tests run)

test-release: ## Run full workspace tests plus release-critical evidence checks
	@$(MAKE) -f make/cargo.mk test-all
	@$(MAKE) evidence-battle
	@$(MAKE) evidence-cache
	@$(MAKE) evidence-replay
	@$(MAKE) evidence-compat
	@$(MAKE) evidence-fault
	@$(MAKE) evidence-perf
	@$(MAKE) evidence-consumers
	@$(MAKE) evidence-release-set

contract-all: ## Run all contract suites with evidence foundation verification
	@mkdir -p $(ARTIFACT_ROOT)/contracts/$(RUN_ID)
	@status=0; total=0; passed=0; failed=0; failed_items=""; report_file="$(ARTIFACT_ROOT)/contracts/$(RUN_ID)/contract-all.log"; \
	run_step() { \
		step_name="$$1"; step_cmd="$$2"; \
		total=$$((total + 1)); \
		printf '%s\n' "run: $$step_name" | tee -a "$$report_file"; \
		eval "$$step_cmd" 2>&1 | tee -a "$$report_file"; \
		step_status=$${PIPESTATUS:-$${pipestatus}}; \
		if [ "$$step_status" -eq 0 ]; then \
			passed=$$((passed + 1)); \
		else \
			failed=$$((failed + 1)); \
			failed_items="$$failed_items"$$'\n'"$$step_name"; \
			status=$$step_status; \
		fi; \
		return "$$step_status"; \
	}; \
	: > "$$report_file"; \
	run_step "contracts.run" '$(DEV_TOOL) contracts run' || true; \
	if [ "$$status" -eq 0 ]; then run_step "evidence-schema" '$(MAKE) -s evidence-schema' || true; fi; \
	if [ "$$status" -eq 0 ]; then run_step "evidence-registry" '$(MAKE) -s evidence-registry' || true; fi; \
	if [ "$$status" -eq 0 ]; then run_step "evidence-consumers" '$(MAKE) -s evidence-consumers' || true; fi; \
	if [ "$$status" -eq 0 ]; then run_step "evidence-all" '$(MAKE) -s evidence-all' || true; fi; \
	printf '\033[1;36m%s\033[0m total=%s \033[1;32mpassed=%s\033[0m \033[1;31mfailed=%s\033[0m \033[1;33mskipped=%s\033[0m \033[1;35mleaky=%s\033[0m\n' "nextest-summary:" "$$total" "$$passed" "$$failed" "0" "0"; \
	if [ "$$failed" -gt 0 ]; then \
		printf '\033[1;31m%s\033[0m\n' "failed-tests:"; \
		printf '%s\n' "$$failed_items" | sed '/^$$/d' | sed 's/^/  /'; \
	fi; \
	test $$status -eq 0

contracts-all: contract-all ## Compatibility alias for contract-all

release-verify: ## Run release verification
	$(call run_or_fail,Run release verification,$(DEV_TOOL) release verify)

module-hygiene-drift: ## Run module hygiene drift gate
	$(call run_or_fail,Run module hygiene drift gate,cargo test -p bijux-dev-dag --test module_hygiene_governance_contracts -- --nocapture)

docs-truth-drift: ## Run documentation truth-boundary drift gate
	$(call run_or_fail,Run docs truth drift gate,cargo test -p bijux-dev-dag --test docs_truth_drift_contracts -- --nocapture)

repo-deps: ## Run repository dependency checks
	$(call run_or_fail,Run dependency policy suite,$(DEV_TOOL) repo deps)

public-surface: ## Run public API surface checks
	$(call run_or_fail,Run public API surface checks,$(DEV_TOOL) api public-surface)

artifacts-clean: ## Clean generated artifacts via control-plane
	$(call run_or_fail,Clean artifacts via dev command,$(DEV_TOOL) artifacts-clean)

surface-explain: ## Explain public API contract checks
	$(call run_or_fail,Explain public API suite,$(DEV_TOOL) contracts explain --suite public-api)

doctor: ## Run developer health checks
	$(call run_or_fail,Run developer health checks,$(DEV_TOOL) doctor)

benchmark-baseline: ## Record benchmark baseline artifacts
	$(call run_or_fail,Record benchmark baseline,$(DEV_TOOL) benchmark-baseline)

memory-smoke: ## Record memory smoke artifacts
	$(call run_or_fail,Record memory smoke artifact,$(DEV_TOOL) memory-smoke)

artifact-verify: ## Verify artifact reproducibility
	$(call run_or_fail,Verify artifact reproducibility,$(DEV_TOOL) artifact-verify)

ci: release-verify ## Run CI gate

sanity: doctor ## Run local sanity gate

help-contract: ## Print the make contract location
	@echo "See make/CONTRACT.md"

make-target-list: ## Print tracked make targets
	@cat make/target-list.json

help: ## Show available make targets
	@printf '%s\n' "bijux-dag make targets"; \
	printf '%s\n' ""; \
	awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_.-]+:.*## / {printf "  %-22s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: all checks checks-fast checks-all docs security compat golden tests-all test-release contract-all contracts-all
.PHONY: release-verify module-hygiene-drift docs-truth-drift repo-deps public-surface artifacts-clean surface-explain doctor benchmark-baseline
.PHONY: memory-smoke artifact-verify ci sanity help help-contract make-target-list
