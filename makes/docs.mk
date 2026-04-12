# Documentation commands that keep generated files under `artifacts/`.

# Prefer the virtualenv binary and fall back to PATH.
ACT              ?= $(VENV)/bin
MKDOCS_BIN_CAND  ?= $(ACT)/mkdocs
MKDOCS_BIN       = $(shell test -x "$(MKDOCS_BIN_CAND)" && printf "%s" "$(MKDOCS_BIN_CAND)" || command -v mkdocs)
DOCS_PYTHON_BIN_CAND ?= $(ACT)/python
DOCS_PYTHON_BIN      = $(shell test -x "$(DOCS_PYTHON_BIN_CAND)" && printf "%s" "$(DOCS_PYTHON_BIN_CAND)" || command -v python3 || command -v python)
MKDOCS_CFG       ?= mkdocs.yml
DOCS_REQUIREMENTS ?= requirements-docs.txt

# Keep documentation build outputs and caches under `artifacts/`.
DOCS_SITE_DIR    ?= artifacts/docs/site
DOCS_CACHE_DIR   ?= artifacts/docs/.cache
DOCS_CONTRACT_DIR ?= $(DOCS_SITE_DIR)/contracts

ENABLE_SOCIAL_CARDS ?= false
SITE_URL            ?= http://127.0.0.1:8000/
DOCS_HOST           ?= 127.0.0.1
DOCS_PORT           ?= 8000

# macOS dynamic loader hints for Homebrew-based environments.
ifeq ($(shell uname -s),Darwin)
  BREW_PREFIX   := $(shell command -v brew >/dev/null 2>&1 && brew --prefix)
  LIBFFI_PREFIX := $(shell test -n "$(BREW_PREFIX)" && brew --prefix libffi)
  DOCS_ENV      := DISABLE_MKDOCS_2_WARNING=true DYLD_FALLBACK_LIBRARY_PATH="$(BREW_PREFIX)/lib:$(LIBFFI_PREFIX)/lib:$$DYLD_FALLBACK_LIBRARY_PATH"
else
  DOCS_ENV      := DISABLE_MKDOCS_2_WARNING=true
endif

.PHONY: docs docs-clean docs-serve docs-deploy docs-check docs-hygiene docs-require docs-install docs-cli-structure-check docs-dag-structure-check docs-package-surface-check docs-navigation-check

##@ Documentation
docs-require: ## Verify the documentation toolchain and configuration
	@$(call require_tool,$(MKDOCS_BIN))
	@$(call require_file,$(MKDOCS_CFG))
	@$(call require_file,$(DOCS_REQUIREMENTS))

docs-install: ## Install the documentation toolchain dependencies
	@echo "Installing documentation dependencies from $(DOCS_REQUIREMENTS)"
	@"$(DOCS_PYTHON_BIN)" -m pip install -r "$(DOCS_REQUIREMENTS)"

docs: docs-clean docs-require ## Build documentation into artifacts/docs/site
	@echo "Building documentation"
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" build --strict --config-file "$(MKDOCS_CFG)" --site-dir "$(DOCS_SITE_DIR)"
	@$(MAKE) docs-hygiene
	@echo "Documentation build complete"

docs-serve: docs-require ## Serve documentation locally with automatic reloads
	@HOST=$${HOST:-$(DOCS_HOST)}; PORT=$${PORT:-$(DOCS_PORT)}; \
	  if command -v lsof >/dev/null 2>&1; then \
	    while lsof -tiTCP:$$PORT -sTCP:LISTEN >/dev/null 2>&1; do PORT=$$((PORT+1)); done; \
	  fi; \
	  echo "Serving documentation on http://$$HOST:$$PORT/"; \
	  mkdir -p "$(DOCS_CACHE_DIR)"; \
	  XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) SITE_URL=http://$$HOST:$$PORT/ \
	    "$(MKDOCS_BIN)" serve --config-file "$(MKDOCS_CFG)" --dev-addr $$HOST:$$PORT

docs-deploy: docs-require ## Deploy documentation to GitHub Pages
	@echo "Deploying documentation to GitHub Pages"
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" gh-deploy --strict --config-file "$(MKDOCS_CFG)"

docs-check: docs-require ## Verify that documentation builds without errors
	@echo "Checking documentation build integrity"
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" build --strict --quiet \
	    --config-file "$(MKDOCS_CFG)" \
	    --site-dir "$(DOCS_SITE_DIR)"
	@$(MAKE) docs-hygiene
	@$(MAKE) docs-cli-structure-check
	@$(MAKE) docs-dag-structure-check
	@$(MAKE) docs-package-surface-check
	@$(MAKE) docs-navigation-check
	@echo "Documentation passes build checks"

docs-clean: ## Remove generated documentation outputs
	@echo "Cleaning documentation build artifacts"
	@rm -rf "$(DOCS_SITE_DIR)" "$(DOCS_CACHE_DIR)" site .cache

docs-hygiene: ## Verify that documentation outputs stay out of the repo root
	@test ! -e "site"   || (echo "ERROR: root 'site/' is forbidden"; exit 1)
	@test ! -e ".cache" || (echo "ERROR: root '.cache/' is forbidden"; exit 1)
	@test ! -d "docs/artifacts" || (echo "ERROR: generated 'docs/artifacts' is forbidden"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/schemas/output-envelope-v1.schema.json" || (echo "ERROR: published contract schema copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/schemas/error-envelope-v1.schema.json" || (echo "ERROR: published contract error schema copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/schemas/plugin-manifest-v2.schema.json" || (echo "ERROR: published contract plugin schema copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/official_product_namespace_registry.json" || (echo "ERROR: published contract registry copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/product_mount_metadata_contract.json" || (echo "ERROR: published contract mount metadata copy is missing"; exit 1)
	@echo "Docs hygiene OK"

docs-cli-structure-check: ## Enforce canonical CLI handbook structure (5x10 pages)
	@for d in foundation architecture interfaces operations quality; do \
	  test -d "docs/bijux-cli/$$d" || (echo "ERROR: missing docs/bijux-cli/$$d" && exit 1); \
	  count=$$(find "docs/bijux-cli/$$d" -mindepth 1 -maxdepth 1 -type f -name '*.md' | wc -l | tr -d ' '); \
	  test "$$count" = "10" || (echo "ERROR: docs/bijux-cli/$$d must contain exactly 10 markdown pages (found $$count)" && exit 1); \
	done
	@test -d "docs/bijux-cli/packages" || (echo "ERROR: missing docs/bijux-cli/packages" && exit 1)
	@test -f "docs/bijux-cli/packages/bijux-cli/index.md" || (echo "ERROR: missing docs/bijux-cli/packages/bijux-cli/index.md" && exit 1)
	@test -f "docs/bijux-cli/packages/bijux-cli-python/index.md" || (echo "ERROR: missing docs/bijux-cli/packages/bijux-cli-python/index.md" && exit 1)
	@echo "CLI docs structure OK"

docs-dag-structure-check: ## Enforce canonical DAG handbook structure (5x10 pages)
	@for d in foundation architecture interfaces operations quality; do \
	  test -d "docs/bijux-dag/$$d" || (echo "ERROR: missing docs/bijux-dag/$$d" && exit 1); \
	  count=$$(find "docs/bijux-dag/$$d" -mindepth 1 -maxdepth 1 -type f -name '*.md' | wc -l | tr -d ' '); \
	  test "$$count" = "10" || (echo "ERROR: docs/bijux-dag/$$d must contain exactly 10 markdown pages (found $$count)" && exit 1); \
	done
	@test -d "docs/bijux-dag/packages" || (echo "ERROR: missing docs/bijux-dag/packages" && exit 1)
	@test -f "docs/bijux-dag/packages/bijux-dag-core/index.md" || (echo "ERROR: missing docs/bijux-dag/packages/bijux-dag-core/index.md" && exit 1)
	@test -f "docs/bijux-dag/packages/bijux-dag-runtime/index.md" || (echo "ERROR: missing docs/bijux-dag/packages/bijux-dag-runtime/index.md" && exit 1)
	@test -f "docs/bijux-dag/packages/bijux-dag-app/index.md" || (echo "ERROR: missing docs/bijux-dag/packages/bijux-dag-app/index.md" && exit 1)
	@test -f "docs/bijux-dag/packages/bijux-dag-cli/index.md" || (echo "ERROR: missing docs/bijux-dag/packages/bijux-dag-cli/index.md" && exit 1)
	@test -f "docs/bijux-dag/packages/bijux-dag-artifacts/index.md" || (echo "ERROR: missing docs/bijux-dag/packages/bijux-dag-artifacts/index.md" && exit 1)
	@test -f "docs/bijux-dag/packages/bijux-dag-testkit/index.md" || (echo "ERROR: missing docs/bijux-dag/packages/bijux-dag-testkit/index.md" && exit 1)
	@echo "DAG docs structure OK"

docs-package-surface-check: ## Verify repository and maintainer package surfaces exist
	@test -f "docs/bijux-core/packages/index.md" || (echo "ERROR: missing docs/bijux-core/packages/index.md" && exit 1)
	@test -f "docs/bijux-dev/packages/bijux-dev/index.md" || (echo "ERROR: missing docs/bijux-dev/packages/bijux-dev/index.md" && exit 1)
	@echo "Package surface docs OK"

docs-navigation-check: ## Verify shared chrome and handbook/package tabs are rendered
	@rg -q 'bijux-hub-strip' "$(DOCS_SITE_DIR)/index.html" || (echo "ERROR: shared Bijux hub strip is missing" && exit 1)
	@rg -q 'bijux-site-tabs' "$(DOCS_SITE_DIR)/index.html" || (echo "ERROR: shared site tabs are missing" && exit 1)
	@rg -q 'Repository Handbook' "$(DOCS_SITE_DIR)/index.html" || (echo "ERROR: Repository Handbook tab label is missing" && exit 1)
	@rg -q 'CLI Handbook' "$(DOCS_SITE_DIR)/index.html" || (echo "ERROR: CLI Handbook tab label is missing" && exit 1)
	@rg -q 'DAG Handbook' "$(DOCS_SITE_DIR)/index.html" || (echo "ERROR: DAG Handbook tab label is missing" && exit 1)
	@rg -q 'Maintainer Handbook' "$(DOCS_SITE_DIR)/index.html" || (echo "ERROR: Maintainer Handbook tab label is missing" && exit 1)
	@rg -q '/bijux-core/packages/' "$(DOCS_SITE_DIR)" || (echo "ERROR: repository package tab is missing" && exit 1)
	@rg -q '/bijux-cli/packages/bijux-cli-python/' "$(DOCS_SITE_DIR)" || (echo "ERROR: CLI Python package tab is missing" && exit 1)
	@rg -q '/bijux-dag/packages/bijux-dag-runtime/' "$(DOCS_SITE_DIR)" || (echo "ERROR: DAG runtime package tab is missing" && exit 1)
	@rg -q '/bijux-dev/packages/bijux-dev/' "$(DOCS_SITE_DIR)" || (echo "ERROR: maintainer package tab is missing" && exit 1)
	@rg -q 'data-bijux-course-strip' "$(DOCS_SITE_DIR)/bijux-cli/index.html" || (echo "ERROR: fourth-row handbook section strip is missing" && exit 1)
	@rg -q 'Foundation' "$(DOCS_SITE_DIR)/bijux-cli/index.html" || (echo "ERROR: CLI section strip labels are missing" && exit 1)
	@rg -q 'Governance' "$(DOCS_SITE_DIR)/bijux-dev/index.html" || (echo "ERROR: maintainer fourth-row navigation labels are missing" && exit 1)
	@echo "Docs navigation OK"
