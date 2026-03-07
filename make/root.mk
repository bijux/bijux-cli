SHELL := /bin/bash

ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
CARGO_TARGET_DIR ?= $(CURDIR)/artifacts/target
NEXTEST_CACHE_DIR ?= $(CURDIR)/artifacts/target/nextest

DEV_TOOL := CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" RUSTFLAGS="-Aunused-crate-dependencies" cargo run -p bijux-dev-dag --bin bijux-dev-dag --
RUN_DIR := artifacts/runs
REPORT_DIR := artifacts/reports
TARGET_DIR := artifacts/target
CONTRACT_DIR := artifacts/contracts
OUTPUT_PATHS := $(RUN_DIR) $(REPORT_DIR) $(TARGET_DIR) $(CONTRACT_DIR)

include make/macros.mk
include make/cargo.mk

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
	$(call run_or_fail,Run all checks,$(DEV_TOOL) checks run)

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

contract-all: ## Run all contract suites with evidence foundation verification
	$(call run_or_fail,Run all contract suites,$(DEV_TOOL) contracts run)
	$(call run_or_fail,Run evidence foundation verification,$(DEV_TOOL) verify evidence-foundation)

contracts-all: contract-all ## Compatibility alias for contract-all

evidence-all: ## Run evidence governance verification entrypoint
	$(call run_or_fail,Run evidence foundation verification,$(DEV_TOOL) verify evidence-foundation)

release-verify: ## Run release verification
	$(call run_or_fail,Run release verification,$(DEV_TOOL) release verify)

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

.PHONY: all checks checks-fast checks-all docs security compat golden tests-all contract-all contracts-all evidence-all
.PHONY: release-verify repo-deps public-surface artifacts-clean surface-explain doctor benchmark-baseline
.PHONY: memory-smoke artifact-verify ci sanity help help-contract make-target-list
