# Documentation (hygienic: nothing emitted under repo root/docs by default)

# Prefer venv binary; fall back to PATH
ACT              ?= $(VENV)/bin
MKDOCS_BIN_CAND  ?= $(ACT)/mkdocs
MKDOCS_BIN       := $(shell test -x "$(MKDOCS_BIN_CAND)" && printf "%s" "$(MKDOCS_BIN_CAND)" || command -v mkdocs)
MKDOCS_CFG       ?= mkdocs.yml

# Keep build/cache strictly under artifacts/
DOCS_GEN_DIR     ?= artifacts/docs/docs
DOCS_SITE_DIR    ?= artifacts/docs/site
DOCS_CACHE_DIR   ?= artifacts/docs/.cache

ENABLE_SOCIAL_CARDS ?= false
SITE_URL            ?= http://127.0.0.1:8000/
DOCS_HOST           ?= 127.0.0.1
DOCS_PORT           ?= 8000

PY ?= python3

# macOS dynamic loader hints (Homebrew only)
ifeq ($(shell uname -s),Darwin)
  BREW_PREFIX   := $(shell command -v brew >/dev/null 2>&1 && brew --prefix)
  LIBFFI_PREFIX := $(shell test -n "$(BREW_PREFIX)" && brew --prefix libffi)
  DOCS_ENV      := DYLD_FALLBACK_LIBRARY_PATH="$(BREW_PREFIX)/lib:$(LIBFFI_PREFIX)/lib:$$DYLD_FALLBACK_LIBRARY_PATH"
else
  DOCS_ENV      :=
endif

# Guardrails
ifeq ($(strip $(MKDOCS_BIN)),)
  $(error mkdocs not found. Activate your venv or install dev deps)
endif
ifeq ($(wildcard $(MKDOCS_CFG)),)
  $(error mkdocs config '$(MKDOCS_CFG)' not found)
endif

.PHONY: docs docs-clean docs-serve docs-deploy docs-check docs-hygiene docs-prep

docs: docs-clean docs-prep
	@echo "Building documentation"
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" build --strict --config-file "$(MKDOCS_CFG)" --site-dir "$(DOCS_SITE_DIR)"
	@$(MAKE) docs-hygiene
	@echo "Documentation build complete"

docs-serve: docs-prep
	@HOST=$${HOST:-$(DOCS_HOST)}; PORT=$${PORT:-$(DOCS_PORT)}; \
	  if command -v lsof >/dev/null 2>&1; then \
	    while lsof -tiTCP:$$PORT -sTCP:LISTEN >/dev/null 2>&1; do PORT=$$((PORT+1)); done; \
	  fi; \
	  echo "Serving documentation on http://$$HOST:$$PORT/"; \
	  mkdir -p "$(DOCS_CACHE_DIR)"; \
	  XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) SITE_URL=http://$$HOST:$$PORT/ \
	    "$(MKDOCS_BIN)" serve --config-file "$(MKDOCS_CFG)" --dev-addr $$HOST:$$PORT

docs-deploy:
	@echo "Deploying documentation to GitHub Pages"
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" gh-deploy --strict --config-file "$(MKDOCS_CFG)"

docs-check:
	@echo "Checking documentation build integrity"
	@mkdir -p "$(DOCS_CACHE_DIR)"
	@XDG_CACHE_HOME="$(DOCS_CACHE_DIR)" $(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) \
	  "$(MKDOCS_BIN)" build --strict --quiet \
	    --config-file "$(MKDOCS_CFG)" \
	    --site-dir "$(DOCS_SITE_DIR)"
	@$(MAKE) docs-hygiene
	@echo "Documentation passes build checks"

docs-prep:
	@echo "Preparing docs → $(DOCS_GEN_DIR)"
	@rm -rf "$(DOCS_GEN_DIR)"
	@mkdir -p "$(DOCS_GEN_DIR)"
	@if [ -d docs/ADR ]; then rsync -a --delete docs/ADR/ "$(DOCS_GEN_DIR)/ADR/"; fi
	@if [ -d docs/assets ]; then rsync -a --delete docs/assets/ "$(DOCS_GEN_DIR)/assets/"; fi
	@GEN_FILES_DISK_FALLBACK=1 GEN_FILES_DISK_DIR="$(DOCS_GEN_DIR)" \
	  $(PY) scripts/docs_builder/mkdocs_manager.py
	@test -f "$(DOCS_GEN_DIR)/nav.md" || (echo "ERROR: nav.md was not generated under $(DOCS_GEN_DIR)"; exit 1)
	@echo "[docs-prep] nav.md present at $(DOCS_GEN_DIR)/nav.md"

docs-clean:
	@echo "Cleaning documentation build artifacts"
	@rm -rf "$(DOCS_SITE_DIR)" artifacts/docs/.cache "$(DOCS_GEN_DIR)" site .cache

docs-hygiene:
	@test ! -e "site"   || (echo "ERROR: root 'site/' is forbidden"; exit 1)
	@test ! -e ".cache" || (echo "ERROR: root '.cache/' is forbidden"; exit 1)
	@test ! -d "docs/artifacts" || (echo "ERROR: generated 'docs/artifacts' is forbidden; use docs-prep→$(DOCS_GEN_DIR)"; exit 1)
	@test ! -d "docs/reference" || (echo "ERROR: generated 'docs/reference' is forbidden; use docs-prep→$(DOCS_GEN_DIR)"; exit 1)
	@echo "Docs hygiene OK"

##@ Documentation
docs:         ## Build documentation (mkdocs --strict) to artifacts/docs/site
docs-serve:   ## Serve documentation locally (auto-reload; no disk generation)
docs-deploy:  ## Deploy documentation to GitHub Pages (strict)
docs-check:   ## Validate documentation builds without errors
docs-clean:   ## Remove generated documentation artifacts
docs-prep:    ## Generate markdown to disk under artifacts/docs/generated (optional)
docs-hygiene: ## Fail if root 'site/' or '.cache/' or generated under docs/
