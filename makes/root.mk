ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

include $(ROOT_MK_DIR)/_macro.mk
include $(ROOT_MK_DIR)/_internal.mk
include $(ROOT_MK_DIR)/rust.mk
include $(ROOT_MK_DIR)/python.mk
include $(ROOT_MK_DIR)/docs.mk
include $(ROOT_MK_DIR)/gh.mk
include $(ROOT_MK_DIR)/dag.mk

workspace-verify: ## Verify unified workspace layout contracts
	 test -p bijux-core-dev --test source_layout_guardrails -- --nocapture

repo-verify: workspace-verify ## Verify workspace layout and cargo health from repository root
	@cargo metadata --no-deps --format-version 1 >/dev/null
	@cargo check --workspace --all-targets -q
