BIJUX_PY_WORKSPACE_DIR ?= $(abspath $(PROJECT_DIR)/..)
BIJUX_PY_REPOS ?= bijux-canon bijux-proteomics bijux-pollenomics
BIJUX_PY_SYSTEM_REL ?= shared/bijux-makes-py
BIJUX_PY_LOCAL_REL ?= makes/bijux-py
BIJUX_PY_REQUIRED_FILES ?= api-contract.mk api-freeze.mk api-live-contract.mk api.mk bijux.mk package.mk package-catalog.mk ci/build.mk ci/docs.mk ci/help.mk ci/lint.mk ci/quality.mk ci/sbom.mk ci/security.mk ci/test.mk ci/util.mk repository/config-layout.mk repository/env.mk repository/make-layout.mk repository/publish.mk repository/root.mk root/docs.mk root/env.mk root/lifecycle.mk root/package-dispatch.mk
BIJUX_PY_OPTIONAL_FILES ?=

.PHONY: check-shared-bijux-py

check-shared-bijux-py: ## Verify shared bijux-py make modules match across sibling repositories
	@set -eu; \
	current_repo="$(PROJECT_SLUG)"; \
	workspace_dir="$(BIJUX_PY_WORKSPACE_DIR)"; \
	current_system_dir="$$workspace_dir/$$current_repo/$(BIJUX_PY_SYSTEM_REL)"; \
	current_local_dir="$$workspace_dir/$$current_repo/$(BIJUX_PY_LOCAL_REL)"; \
	compared_repos=0; \
	[ -d "$$current_system_dir" ] || { echo "✘ Missing shared make directory: $$current_system_dir"; exit 2; }; \
	[ -d "$$current_local_dir" ] || { echo "✘ Missing local make directory: $$current_local_dir"; exit 2; }; \
	for file in $(BIJUX_PY_REQUIRED_FILES); do \
	  [ -f "$$current_system_dir/$$file" ] || { echo "✘ Missing $$current_system_dir/$$file"; exit 2; }; \
	  [ -f "$$current_local_dir/$$file" ] || { echo "✘ Missing $$current_local_dir/$$file"; exit 2; }; \
	  cmp -s "$$current_system_dir/$$file" "$$current_local_dir/$$file" || { echo "✘ Local bijux-py drift: $$file differs between $(BIJUX_PY_SYSTEM_REL) and $(BIJUX_PY_LOCAL_REL) in $$current_repo"; exit 1; }; \
	done; \
	for file in $(BIJUX_PY_OPTIONAL_FILES); do \
	  if [ -f "$$current_system_dir/$$file" ] && [ -f "$$current_local_dir/$$file" ]; then \
	    cmp -s "$$current_system_dir/$$file" "$$current_local_dir/$$file" || { echo "✘ Local bijux-py drift: $$file differs between $(BIJUX_PY_SYSTEM_REL) and $(BIJUX_PY_LOCAL_REL) in $$current_repo"; exit 1; }; \
	  fi; \
	done; \
	for repo in $(BIJUX_PY_REPOS); do \
	  [ "$$repo" = "$$current_repo" ] && continue; \
	  other_system_dir="$$workspace_dir/$$repo/$(BIJUX_PY_SYSTEM_REL)"; \
	  if [ ! -d "$$other_system_dir" ]; then \
	    echo "→ Skipping sibling shared make check for $$repo; $$other_system_dir is not present"; \
	    continue; \
	  fi; \
	  compared_repos=$$((compared_repos + 1)); \
	  for file in $(BIJUX_PY_REQUIRED_FILES); do \
	    [ -f "$$other_system_dir/$$file" ] || { echo "✘ Missing $$other_system_dir/$$file"; exit 2; }; \
	    cmp -s "$$current_system_dir/$$file" "$$other_system_dir/$$file" || { echo "✘ Shared make drift: $$file differs between $$current_repo and $$repo"; exit 1; }; \
	  done; \
	  for file in $(BIJUX_PY_OPTIONAL_FILES); do \
	    if [ -f "$$current_system_dir/$$file" ] && [ -f "$$other_system_dir/$$file" ]; then \
	      cmp -s "$$current_system_dir/$$file" "$$other_system_dir/$$file" || { echo "✘ Shared make drift: $$file differs between $$current_repo and $$repo"; exit 1; }; \
	    fi; \
	  done; \
	done; \
	if [ "$$compared_repos" -eq 0 ]; then \
	  echo "✔ bijux-py modules are self-consistent; sibling repositories are not present in $$workspace_dir"; \
	else \
	  echo "✔ bijux-py modules match across $$compared_repos available sibling repositories"; \
	fi
