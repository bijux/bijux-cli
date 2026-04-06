# DAG integration targets for the unified bijux-core make surface.

DAG_MAKEFILE ?= makes/dag/root.mk

.PHONY: dag-help dag-check dag-test dag-test-all dag-clippy dag-coverage dag-contracts dag-release
dag-help: ## Show dag-specific make targets
	@$(MAKE) -f "$(DAG_MAKEFILE)" help

dag-check: ## Run DAG workspace checks
	@$(MAKE) -f "$(DAG_MAKEFILE)" check

dag-test: ## Run DAG fast tests
	@$(MAKE) -f "$(DAG_MAKEFILE)" test

dag-test-all: ## Run DAG full tests including ignored suites
	@$(MAKE) -f "$(DAG_MAKEFILE)" test-all

dag-clippy: ## Run DAG lint checks
	@$(MAKE) -f "$(DAG_MAKEFILE)" clippy

dag-coverage: ## Run DAG coverage suite
	@$(MAKE) -f "$(DAG_MAKEFILE)" coverage

dag-contracts: ## Run DAG contract and evidence checks
	@$(MAKE) -f "$(DAG_MAKEFILE)" contract-all

dag-release: ## Run DAG release verification
	@$(MAKE) -f "$(DAG_MAKEFILE)" release-verify
