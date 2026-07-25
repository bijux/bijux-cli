# Documentation commands that keep generated files under `artifacts/`.

# Prefer the virtualenv binary and fall back to PATH.
ACT              ?= $(VENV)/bin
MKDOCS_BIN_CAND  ?= $(ACT)/mkdocs
MKDOCS_BIN       = $(shell test -x "$(MKDOCS_BIN_CAND)" && printf "%s" "$(MKDOCS_BIN_CAND)" || command -v mkdocs)
DOCS_PYTHON_BIN_CAND ?= $(ACT)/python
DOCS_PYTHON_BIN      = $(shell test -x "$(DOCS_PYTHON_BIN_CAND)" && printf "%s" "$(DOCS_PYTHON_BIN_CAND)" || command -v python3 || command -v python)
MKDOCS_CFG       ?= mkdocs.yml
DOCS_REQUIREMENTS ?= configs/docs/requirements-docs.txt

# Keep documentation build outputs and caches under `artifacts/`.
DOCS_SITE_DIR    ?= artifacts/docs/site
DOCS_CACHE_DIR   ?= artifacts/docs/.cache
DOCS_PYCACHE_DIR ?= artifacts/docs/pycache
DOCS_CONTRACT_DIR ?= $(DOCS_SITE_DIR)/contracts

define docs_search_file
if command -v rg >/dev/null 2>&1; then \
  rg -q '$(1)' "$(2)"; \
else \
  grep -q '$(1)' "$(2)"; \
fi
endef

define docs_search_tree
if command -v rg >/dev/null 2>&1; then \
  rg -q '$(1)' "$(2)"; \
else \
  grep -R -q '$(1)' "$(2)"; \
fi
endef

ENABLE_SOCIAL_CARDS ?= false
SITE_URL            ?= http://127.0.0.1:8000/
DOCS_HOST           ?= 127.0.0.1
DOCS_PORT           ?= 8000

# macOS dynamic loader hints for Homebrew-based environments.
ifeq ($(shell uname -s),Darwin)
  BREW_PREFIX   := $(shell command -v brew >/dev/null 2>&1 && brew --prefix)
  LIBFFI_PREFIX := $(shell test -n "$(BREW_PREFIX)" && brew --prefix libffi)
  DOCS_ENV      := DISABLE_MKDOCS_2_WARNING=true PYTHONPYCACHEPREFIX="$(abspath $(DOCS_PYCACHE_DIR))" DYLD_FALLBACK_LIBRARY_PATH="$(BREW_PREFIX)/lib:$(LIBFFI_PREFIX)/lib:$$DYLD_FALLBACK_LIBRARY_PATH"
else
  DOCS_ENV      := DISABLE_MKDOCS_2_WARNING=true PYTHONPYCACHEPREFIX="$(abspath $(DOCS_PYCACHE_DIR))"
endif

.PHONY: docs docs-clean docs-serve docs-deploy docs-check docs-hygiene docs-require docs-install docs-publication-check docs-navigation-check docs-mermaid-check readme-links-check sync-readme-links

docs docs-serve docs-deploy docs-check docs-install docs-require: | bootstrap

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
	@$(MAKE) --no-print-directory bijux-docs-sync
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" build --strict --config-file "$(MKDOCS_CFG)" --site-dir "$(DOCS_SITE_DIR)"
	@$(MAKE) docs-hygiene
	@echo "Documentation build complete"

docs-serve: docs-require ## Serve documentation locally with automatic reloads
	@$(MAKE) --no-print-directory bijux-docs-sync
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
	@$(MAKE) --no-print-directory docs-install
	@$(MAKE) --no-print-directory bijux-docs-check
	@$(MAKE) --no-print-directory check-badges
	@$(MAKE) --no-print-directory readme-links-check
	@$(MAKE) --no-print-directory docs-mermaid-check
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" build --strict --quiet \
	    --config-file "$(MKDOCS_CFG)" \
	    --site-dir "$(DOCS_SITE_DIR)"
	@$(MAKE) docs-hygiene
	@$(MAKE) docs-publication-check
	@$(MAKE) docs-navigation-check
	@echo "Documentation passes build checks"

docs-mermaid-check: ## Reject Mermaid identifiers that conflict with diagram syntax
	@"$(DOCS_PYTHON_BIN)" docs/automation/mermaid_sanity.py docs

readme-links-check: ## Verify README links use valid public destinations
	@"$(DOCS_PYTHON_BIN)" docs/automation/readme_links.py check

sync-readme-links: ## Replace local README links with canonical public destinations
	@"$(DOCS_PYTHON_BIN)" docs/automation/readme_links.py sync

docs-clean: ## Remove generated documentation outputs
	@echo "Cleaning documentation build artifacts"
	@rm -rf "$(DOCS_SITE_DIR)" "$(DOCS_CACHE_DIR)" site .cache

docs-hygiene: ## Verify that documentation outputs stay out of the repo root
	@test ! -e "site"   || (echo "ERROR: root 'site/' is forbidden"; exit 1)
	@test ! -e ".cache" || (echo "ERROR: root '.cache/' is forbidden"; exit 1)
	@test ! -d "docs/artifacts" || (echo "ERROR: generated 'docs/artifacts' is forbidden"; exit 1)
	@leaked=$$(find docs crates \
	  -path '*/.venv' -prune -o \
	  -path '*/.venv*' -prune -o \
	  \( -type d \( -name '__pycache__' -o -name '.pytest_cache' \) \
	     -o -type f \( -name '*.pyc' -o -name '*.pyo' \) \) -print); \
	  test -z "$$leaked" || \
	    (echo "ERROR: Python caches must be written under artifacts/:"; echo "$$leaked"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/schemas/output-envelope-v1.schema.json" || (echo "ERROR: published contract schema copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/schemas/error-envelope-v1.schema.json" || (echo "ERROR: published contract error schema copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/schemas/plugin-manifest-v2.schema.json" || (echo "ERROR: published contract plugin schema copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/official_product_namespace_registry.json" || (echo "ERROR: published contract registry copy is missing"; exit 1)
	@test -f "$(DOCS_CONTRACT_DIR)/product_mount_metadata_contract.json" || (echo "ERROR: published contract mount metadata copy is missing"; exit 1)
	@echo "Docs hygiene OK"

docs-publication-check: ## Enforce the curated public documentation boundary
	@count=$$(awk '/^[[:space:]]+- [^:]+: .*\.md$$/ { count++ } END { print count + 0 }' "$(MKDOCS_CFG)"); \
	  test "$$count" -ge 40 || (echo "ERROR: public navigation is unexpectedly small ($$count pages)" && exit 1); \
	  test "$$count" -le 100 || (echo "ERROR: public navigation exceeds 100 pages ($$count pages)" && exit 1); \
	  echo "Public documentation page budget OK ($$count pages)"
	@deep=$$(find docs/bijux-core docs/bijux-cli docs/bijux-dag docs/bijux-dev \
	  -type f -name '*.md' | awk -F/ 'NF > 4 { print }'); \
	  test -z "$$deep" || (echo "ERROR: product documentation exceeds product/category/page depth:"; echo "$$deep"; exit 1)
	@for path in \
	  docs/bijux-core/foundation/documentation-system.md \
	  docs/bijux-cli/interfaces/cli-surface.md \
	  docs/bijux-dag/foundation/release-boundary.md \
	  docs/bijux-dag/interfaces/generated-cli-reference.md \
	  docs/bijux-dag/interfaces/reproducibility-model.md \
	  docs/bijux-dag/operations/security-isolation-truth.md \
	  docs/bijux-dag/quality/known-limitations.md \
	  docs/bijux-dev/operations/repository-gates.md; do \
	  test -f "$$path" || (echo "ERROR: missing public authority page $$path" && exit 1); \
	done
	@for path in /spec/ /reports/; do \
	  grep -Fqx "  $$path" "$(MKDOCS_CFG)" || \
	    (echo "ERROR: internal documentation boundary $${path} is not excluded" && exit 1); \
	done
	@core_pages=$$(awk '/^  !\/bijux-core\// { path=$$0; sub(/^  !\//, "docs/", path); print path }' "$(MKDOCS_CFG)"); \
	  stock=""; \
	  if [ -n "$$core_pages" ]; then \
	    stock=$$(printf '%s\n' "$$core_pages" | xargs rg -n \
	      '^## (Visual Summary|Reader Shortcut|Continue Reading|Next Reads|Reading Rule|What This Page Is Not Saying|Open Next)$$|^Use this page when' || true); \
	  fi; \
	  test -z "$$stock" || (echo "ERROR: published repository handbook uses stock presentation prose:"; echo "$$stock"; exit 1)
	@echo "Documentation publication boundary OK"

docs-navigation-check: ## Verify shared chrome and handbook/package tabs are rendered
	@$(call docs_search_file,bijux-hub-strip,$(DOCS_SITE_DIR)/index.html) || (echo "ERROR: shared Bijux hub strip is missing" && exit 1)
	@$(call docs_search_file,bijux-site-tabs,$(DOCS_SITE_DIR)/index.html) || (echo "ERROR: shared site tabs are missing" && exit 1)
	@$(call docs_search_file,Repository Handbook,$(DOCS_SITE_DIR)/index.html) || (echo "ERROR: Repository Handbook tab label is missing" && exit 1)
	@$(call docs_search_file,CLI Handbook,$(DOCS_SITE_DIR)/index.html) || (echo "ERROR: CLI Handbook tab label is missing" && exit 1)
	@$(call docs_search_file,DAG Handbook,$(DOCS_SITE_DIR)/index.html) || (echo "ERROR: DAG Handbook tab label is missing" && exit 1)
	@$(call docs_search_file,Maintainer Handbook,$(DOCS_SITE_DIR)/index.html) || (echo "ERROR: Maintainer Handbook tab label is missing" && exit 1)
	@$(call docs_search_tree,/bijux-core/packages/,$(DOCS_SITE_DIR)) || (echo "ERROR: repository package tab is missing" && exit 1)
	@$(call docs_search_tree,/bijux-cli/packages/bijux-cli-python/,$(DOCS_SITE_DIR)) || (echo "ERROR: CLI Python package tab is missing" && exit 1)
	@$(call docs_search_tree,/bijux-dag/packages/bijux-dag-runtime/,$(DOCS_SITE_DIR)) || (echo "ERROR: DAG runtime package tab is missing" && exit 1)
	@$(call docs_search_tree,/bijux-dev/packages/bijux-dev/,$(DOCS_SITE_DIR)) || (echo "ERROR: maintainer package tab is missing" && exit 1)
	@$(call docs_search_file,data-bijux-detail-strip,$(DOCS_SITE_DIR)/bijux-cli/index.html) || (echo "ERROR: handbook program strip is missing" && exit 1)
	@$(call docs_search_file,Product,$(DOCS_SITE_DIR)/bijux-cli/index.html) || (echo "ERROR: CLI product navigation is missing" && exit 1)
	@$(call docs_search_file,Architecture,$(DOCS_SITE_DIR)/bijux-core/index.html) || (echo "ERROR: repository architecture navigation is missing" && exit 1)
	@$(call docs_search_file,Build System,$(DOCS_SITE_DIR)/bijux-dev/index.html) || (echo "ERROR: maintainer build navigation is missing" && exit 1)
	@"$(DOCS_PYTHON_BIN)" docs/automation/navigation_sanity.py "$(DOCS_SITE_DIR)"
	@echo "Docs navigation OK"
