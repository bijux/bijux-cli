SHELL := /bin/bash

.PHONY: help all test checks checks-fast checks-all tests-all contracts-all release-verify
.PHONY: lint docs fmt security compat golden repo-deps public-surface ci artifacts-clean surface-explain doctor sanity

DEV_TOOL := cargo run -p bijux-dev-dag --
RUN_DIR := artifacts/runs
REPORT_DIR := artifacts/reports
TARGET_DIR := artifacts/target
CONTRACT_DIR := artifacts/contracts
OUTPUT_PATHS := $(RUN_DIR) $(REPORT_DIR) $(TARGET_DIR) $(CONTRACT_DIR)

# Standard failure guard for wrapper targets.
define run_or_fail
	@echo "--> $(1)"
	@$(2) || (\
		echo "--> failed: $(1)" >&2; \
		echo "--> inspect artifacts: $(OUTPUT_PATHS)" >&2; \
		echo "--> pass --report <path> to write machine-readable output" >&2; \
		exit 1)
endef

help:
	@echo "Bijux-dag project wrapper targets"
	@echo "artifact outputs: $(OUTPUT_PATHS)"
	@echo ""
	@echo "Development:"
	@echo "  test             - Run workspace tests via bijux-dev-dag tests run"
	@echo "  checks           - Run checks suites via bijux-dev-dag checks run"
	@echo "  checks-fast      - Run checks excluding slow suites"
	@echo "  checks-all       - Run all checks suites"
	@echo "  lint             - Legacy alias for bijux-dev-dag checks run"
	@echo "  sanity           - Legacy alias for bijux-dev-dag doctor"
	@echo "  doctor           - Run developer diagnostics via bijux-dev-dag doctor"
	@echo ""
	@echo "Contracts and quality:"
	@echo "  tests-all        - Run all dev tests suites"
	@echo "  contracts-all    - Run all contract suites"
	@echo "  compat           - Run compat contracts"
	@echo "  golden           - Run runtime golden contracts"
	@echo "  public-surface   - Run public API surface contract"
	@echo "  surface-explain  - Explain public-api suite"
	@echo ""
	@echo "Release and governance:"
	@echo "  release-verify   - Run bijux-dev-dag release verify"
	@echo "  repo-deps        - Run bijux-dev-dag repo deps"
	@echo "  ci               - Legacy alias for bijux-dev-dag release verify"
	@echo ""
	@echo "Maintenance:"
	@echo "  fmt              - Format workspace code"
	@echo "  security         - Run security checks"
	@echo "  artifacts-clean  - Remove artifacts"
	@echo "  docs             - Run docs checks"

all: test

test:
	$(call run_or_fail,Run repository tests,$(DEV_TOOL) tests run)

checks:
	$(call run_or_fail,Run default checks,$(DEV_TOOL) checks run --domain policy --domain quality --domain style --domain supply-chain)

checks-fast:
	$(call run_or_fail,Run fast checks,$(DEV_TOOL) checks run)

checks-all:
	$(call run_or_fail,Run all checks,$(DEV_TOOL) checks run)

lint: checks

docs:
	$(call run_or_fail,Run documentation checks,$(DEV_TOOL) docs run)

fmt:
	$(call run_or_fail,Run formatter,$(DEV_TOOL) fmt)

security:
	$(call run_or_fail,Run security checks,$(DEV_TOOL) checks run --domain supply-chain)

compat:
	$(call run_or_fail,Run compatibility contracts,$(DEV_TOOL) contracts run --domain compat)

golden:
	$(call run_or_fail,Run runtime golden contract,$(DEV_TOOL) contracts run --domain runtime)

tests-all:
	$(call run_or_fail,Run all test suites,$(DEV_TOOL) tests run)

contracts-all:
	$(call run_or_fail,Run all contract suites,$(DEV_TOOL) contracts run)

release-verify:
	$(call run_or_fail,Run release verification,$(DEV_TOOL) release verify)

repo-deps:
	$(call run_or_fail,Run dependency policy suite,$(DEV_TOOL) repo deps)

public-surface:
	$(call run_or_fail,Run public API surface checks,$(DEV_TOOL) api public-surface)

ci: release-verify

artifacts-clean:
	$(call run_or_fail,Clean artifacts via dev command,$(DEV_TOOL) artifacts-clean)

surface-explain:
	$(call run_or_fail,Explain public API suite,$(DEV_TOOL) contracts explain --suite public-api)

sanity: doctor

doctor:
	$(call run_or_fail,Run developer health checks,$(DEV_TOOL) doctor)
