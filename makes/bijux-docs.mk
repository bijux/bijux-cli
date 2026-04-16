# Bijux docs shell synchronization and contract enforcement.

PYTHON_BIN                  ?= $(shell command -v python3 2>/dev/null)
BIJUX_DOCS_SYNC_SCRIPT      ?= internal/scripts/sync_bijux_docs.sh
BIJUX_DOCS_SOT_GUARD        ?= internal/scripts/verify_bijux_docs_source_of_truth.sh
BIJUX_DOCS_CONTRACT_GUARD   ?= internal/quality/validate_bijux_docs_contract.py
BIJUX_STD_CHECK_SCRIPT      ?= shared/bijux-checks/check-bijux-std.sh
BIJUX_STD_UPDATE_SCRIPT     ?= shared/bijux-checks/update-bijux-std.sh
BIJUX_STD_REF               ?= main
BIJUX_STD_REMOTE            ?= https://raw.githubusercontent.com/bijux/bijux-std
BIJUX_STD_GIT_URL           ?= https://github.com/bijux/bijux-std.git
BIJUX_STD_UPDATE_CHANNEL    ?= branch
BIJUX_STD_TAG_PATTERN       ?= v*

.PHONY: bijux-docs-sync bijux-docs-check bijux-std bijux-std-checks bijux-std-update shell-sync shell-check

##@ Bijux Docs
bijux-docs-sync: ## Synchronize shared Bijux docs shell into docs assets
	@bash "$(BIJUX_DOCS_SYNC_SCRIPT)"

bijux-docs-check: ## Validate Bijux docs shell contract and drift checks
	@"$(PYTHON_BIN)" "$(BIJUX_DOCS_CONTRACT_GUARD)" .
	@bash "$(BIJUX_DOCS_SOT_GUARD)"

bijux-std-checks: ## Verify shared directories match bijux-std (set BIJUX_STD_REF for pinning)
	@BIJUX_STD_REF="$(BIJUX_STD_REF)" BIJUX_STD_REMOTE="$(BIJUX_STD_REMOTE)" bash "$(BIJUX_STD_CHECK_SCRIPT)"

bijux-std-update: ## Update shared directories from bijux-std (set BIJUX_STD_UPDATE_CHANNEL=branch|tag)
	@BIJUX_STD_REF="$(BIJUX_STD_REF)" BIJUX_STD_GIT_URL="$(BIJUX_STD_GIT_URL)" BIJUX_STD_UPDATE_CHANNEL="$(BIJUX_STD_UPDATE_CHANNEL)" BIJUX_STD_TAG_PATTERN="$(BIJUX_STD_TAG_PATTERN)" bash "$(BIJUX_STD_UPDATE_SCRIPT)"

bijux-std: bijux-std-checks ## Backward-compatible alias

# Backward-compatible aliases.
shell-sync: bijux-docs-sync ## Alias for bijux-docs-sync
shell-check: bijux-docs-check ## Alias for bijux-docs-check
