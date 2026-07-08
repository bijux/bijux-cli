# DAG-facing make targets integrated at repository root.

ARTIFACT_ROOT ?= artifacts
RUN_ID ?= local
CARGO_TARGET_DIR ?= $(CURDIR)/artifacts/target
NEXTEST_CACHE_DIR ?= $(CURDIR)/artifacts/target/nextest
LLVM_PROFILE_FILE ?= $(CURDIR)/artifacts/coverage/profraw/default_%m_%p.profraw

DEV_TOOL := LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" NEXTEST_CACHE_DIR="$(NEXTEST_CACHE_DIR)" RUSTFLAGS="-Aunused-crate-dependencies" cargo run -p bijux-dev --bin bijux-dev-dag --

DAG_OUTPUT_PATHS := artifacts/runs artifacts/reports artifacts/target artifacts/contracts

run_or_fail = @echo "--> $(1)"; @$(2) || (echo "--> failed: $(1)" >&2; echo "--> inspect artifacts: $(DAG_OUTPUT_PATHS)" >&2; exit 1)

.PHONY: dag-help dag-demo dag-check dag-test dag-test-all dag-clippy dag-coverage dag-contracts dag-release
.PHONY: checks checks-fast checks-all contracts-all contract-all release-verify
.PHONY: docs-governance-lint docs-inventory-generate module-hygiene-drift docs-truth-drift
.PHONY: evidence-all evidence-verify evidence-battle evidence-authoring evidence-cache evidence-replay
.PHONY: evidence-compat evidence-fault evidence-perf evidence-compare evidence-schema evidence-registry
.PHONY: evidence-consumers evidence-release-set evidence-report evidence-summary evidence-clean

# Shared gates first; DAG aliases keep the dedicated operator entrypoints.
dag-help: ## Show DAG-oriented make targets
	@printf '%s\n' "DAG targets: dag-demo dag-check dag-test dag-test-all dag-clippy dag-coverage dag-contracts dag-release"

dag-demo: ## Run the canonical retained file-processing DAG proof command
	@"$(ROOT_MK_DIR)/bin/run_file_processing_demo.sh"

dag-check: ## Run shared workspace check gate
	@CARGO_TARGET_DIR="$(CARGO_TARGET_DIR)" cargo check --workspace --all-targets

dag-test: test-release-rs ## Run the required shared Rust release lane

dag-test-all: test-all-rs ## Run shared full Rust test suite

dag-clippy: lint-rs ## Run shared Rust lint gate

dag-coverage: coverage-rs ## Run shared Rust coverage gate

dag-contracts: contract-all ## Run DAG contract and evidence gates

dag-release: release-verify ## Run DAG release verification gate

checks: ## Run default control-plane checks
	$(call run_or_fail,Run default checks,$(DEV_TOOL) checks run --domain policy --domain quality --domain style --domain supply-chain)

checks-fast: ## Run fast control-plane checks
	$(call run_or_fail,Run fast checks,$(DEV_TOOL) checks run)

checks-all: ## Run full control-plane checks
	$(call run_or_fail,Run full checks,$(DEV_TOOL) checks run)

docs-governance-lint: ## Lint docs governance metadata and boundaries
	$(call run_or_fail,Run docs governance lint,$(DEV_TOOL) docs run --domain governance --fail-fast)

docs-inventory-generate: ## Generate docs inventory and consolidation reports
	$(call run_or_fail,Generate docs inventory reports,$(DEV_TOOL) docs-index)

contract-all: ## Run all contract suites with evidence foundation verification
	$(call run_or_fail,Run contracts,$(DEV_TOOL) contracts run)
	$(MAKE) --no-print-directory evidence-schema
	$(MAKE) --no-print-directory evidence-registry
	$(MAKE) --no-print-directory evidence-consumers
	$(MAKE) --no-print-directory evidence-all

contracts-all: contract-all ## Compatibility alias for contract-all

release-verify: ## Run release verification
	$(call run_or_fail,Run release verification,$(DEV_TOOL) release verify)

module-hygiene-drift: ## Run module hygiene drift gate
	$(call run_or_fail,Run module hygiene drift gate,cargo test -p bijux-dev --test module_hygiene_governance_contracts -- --nocapture)

docs-truth-drift: ## Run documentation truth-boundary drift gate
	$(call run_or_fail,Run docs truth drift gate,cargo test -p bijux-dev --test docs_truth_drift_contracts -- --nocapture)

evidence-all: evidence-verify ## Run the canonical evidence verification entrypoint

evidence-verify: ## Run the full evidence verification suite
	$(call run_or_fail,Run evidence foundation verification,$(DEV_TOOL) verify evidence-foundation)

evidence-battle: ## Run battle evidence verification
	$(call run_or_fail,Run battle evidence verification,$(DEV_TOOL) verify evidence-battle)

evidence-authoring: ## Run authoring evidence verification
	$(call run_or_fail,Run authoring evidence verification,$(DEV_TOOL) verify evidence-authoring)

evidence-cache: ## Run cache evidence verification
	$(call run_or_fail,Run cache evidence verification,$(DEV_TOOL) verify evidence-cache)

evidence-replay: ## Run replay evidence verification
	$(call run_or_fail,Run replay evidence verification,$(DEV_TOOL) verify evidence-replay)

evidence-compat: ## Run compatibility evidence verification
	$(call run_or_fail,Run compatibility evidence verification,$(DEV_TOOL) verify evidence-compat)

evidence-fault: ## Run fault evidence verification
	$(call run_or_fail,Run fault evidence verification,$(DEV_TOOL) verify evidence-fault)

evidence-perf: ## Run performance evidence verification
	$(call run_or_fail,Run performance evidence verification,$(DEV_TOOL) verify evidence-perf)

evidence-compare: ## Run comparison evidence verification
	$(call run_or_fail,Run comparison evidence verification,$(DEV_TOOL) verify evidence-compare)

evidence-schema: ## Run evidence schema verification
	$(call run_or_fail,Run evidence schema verification,$(DEV_TOOL) verify evidence-schema)

evidence-registry: ## Run evidence registry verification
	$(call run_or_fail,Run evidence registry verification,$(DEV_TOOL) verify evidence-registry)

evidence-consumers: ## Run evidence consumer verification
	$(call run_or_fail,Run evidence consumer verification,$(DEV_TOOL) verify evidence-consumers)

evidence-release-set: ## Run release evidence set verification
	$(call run_or_fail,Run release evidence set verification,$(DEV_TOOL) verify evidence-release-set)

evidence-report: ## Generate evidence suite summary reports
	$(call run_or_fail,Generate evidence summary reports,$(DEV_TOOL) repo evidence-summary-report)

evidence-summary: evidence-report ## Compatibility alias for evidence-report

evidence-clean: ## Clean generated evidence artifacts
	$(call run_or_fail,Clean generated evidence artifacts,$(DEV_TOOL) artifacts-clean)
