ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

include $(ROOT_MK_DIR)/_macro.mk
BIJUX_MAKES_SHARED_ROOT := $(CURDIR)/.bijux/shared
include $(BIJUX_MAKES_SHARED_ROOT)/bijux-makes/environment.mk
include $(BIJUX_MAKES_SHARED_ROOT)/bijux-makes/guards.mk
include $(ROOT_MK_DIR)/_internal.mk
include $(ROOT_MK_DIR)/rust.mk
include $(ROOT_MK_DIR)/python.mk
include $(ROOT_MK_DIR)/docs.mk
include $(ROOT_MK_DIR)/bijux-docs.mk
include $(ROOT_MK_DIR)/bijux-std.mk
include $(ROOT_MK_DIR)/contracts.mk
include $(ROOT_MK_DIR)/gh.mk
include $(ROOT_MK_DIR)/dag.mk
include $(BIJUX_MAKES_SHARED_ROOT)/bijux-makes-rs/bijux.mk

.PHONY: sync-badges check-badges

sync-badges: ## Render README and docs badge blocks from docs/badges.md
	@"$(DOCS_PYTHON_BIN)" docs/automation/badge_sync.py sync

check-badges: ## Verify generated badge blocks match docs/badges.md
	@"$(DOCS_PYTHON_BIN)" docs/automation/badge_sync.py check

workspace-verify: ## Verify unified workspace layout contracts
	 test -p bijux-dev --test source_layout_guardrails -- --nocapture

repo-verify: workspace-verify ## Verify workspace layout and cargo health from repository root
	@cargo metadata --no-deps --format-version 1 >/dev/null
	@cargo check --workspace --all-targets -q
