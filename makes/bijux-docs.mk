# Shared Bijux docs shell synchronization and contract enforcement.

# Prefer repository-managed Python interpreters so docs contract checks run in
# the same environment where docs dependencies were installed.
PYTHON_BIN ?= $(or $(VENV_PYTHON),$(DOCS_PYTHON_BIN),$(shell command -v python3 2>/dev/null))
BIJUX_DOCS_SYNC_SCRIPT ?= shared/bijux-docs/tooling/scripts/sync_bijux_docs.sh
BIJUX_DOCS_SOT_GUARD ?= shared/bijux-docs/tooling/scripts/verify_bijux_docs_source_of_truth.sh
BIJUX_DOCS_CONTRACT_GUARD ?= shared/bijux-docs/tooling/quality/validate_bijux_docs_contract.py
BIJUX_DOCS_ARTIFACTS_DIR ?= artifacts/bijux-docs
BIJUX_DOCS_LOG_DIR ?= $(BIJUX_DOCS_ARTIFACTS_DIR)/logs
BIJUX_DOCS_PYCACHE_DIR ?= $(BIJUX_DOCS_ARTIFACTS_DIR)/pycache
BIJUX_DOCS_XDG_CACHE_DIR ?= $(BIJUX_DOCS_ARTIFACTS_DIR)/xdg_cache
BIJUX_DOCS_HYPOTHESIS_DIR ?= $(BIJUX_DOCS_ARTIFACTS_DIR)/hypothesis

.PHONY: bijux-docs-sync bijux-docs-check shell-sync shell-check

bijux-docs-sync: ## Synchronize shared Bijux docs shell into docs assets
	@mkdir -p "$(BIJUX_DOCS_LOG_DIR)"
	@bash -o pipefail -c 'bash "$(BIJUX_DOCS_SYNC_SCRIPT)" 2>&1 | tee "$(BIJUX_DOCS_LOG_DIR)/sync.log"'

bijux-docs-check: ## Validate Bijux docs shell contract and drift checks
	@mkdir -p "$(BIJUX_DOCS_LOG_DIR)" "$(BIJUX_DOCS_PYCACHE_DIR)" "$(BIJUX_DOCS_XDG_CACHE_DIR)" "$(BIJUX_DOCS_HYPOTHESIS_DIR)"
	@PYTHONPYCACHEPREFIX="$(abspath $(BIJUX_DOCS_PYCACHE_DIR))" XDG_CACHE_HOME="$(abspath $(BIJUX_DOCS_XDG_CACHE_DIR))" HYPOTHESIS_STORAGE_DIRECTORY="$(abspath $(BIJUX_DOCS_HYPOTHESIS_DIR))" "$(PYTHON_BIN)" "$(BIJUX_DOCS_CONTRACT_GUARD)" . 2>&1 | tee "$(BIJUX_DOCS_LOG_DIR)/contract.log"
	@bash -o pipefail -c 'bash "$(BIJUX_DOCS_SOT_GUARD)" 2>&1 | tee "$(BIJUX_DOCS_LOG_DIR)/source-of-truth.log"'

# Backward-compatible aliases.
shell-sync: bijux-docs-sync ## Alias for bijux-docs-sync
shell-check: bijux-docs-check ## Alias for bijux-docs-check
