# Documentation Configuration

MKDOCS_BIN 			:= $(ACT)/mkdocs
DOCS_SITE_DIR 		?= site
ENABLE_SOCIAL_CARDS ?= 0


ifeq ($(shell uname -s),Darwin)
  BREW_PREFIX := $(shell command -v brew >/dev/null 2>&1 && brew --prefix)
  LIBFFI_PREFIX := $(shell test -n "$(BREW_PREFIX)" && brew --prefix libffi)
  DOCS_ENV := DYLD_FALLBACK_LIBRARY_PATH="$(BREW_PREFIX)/lib:$(LIBFFI_PREFIX)/lib:$$DYLD_FALLBACK_LIBRARY_PATH"
else
  DOCS_ENV :=
endif

ifeq ($(strip $(MKDOCS_BIN)),)
  $(error mkdocs not found. Install dev deps or activate your virtualenv)
endif

.PHONY: docs docs-clean docs-serve docs-deploy docs-check

docs: docs-clean
	@echo "Building documentation"
	@$(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) $(MKDOCS_BIN) build --strict
	@echo "Documentation build complete"

docs-serve:
	@echo "Serving documentation on localhost"
	@$(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) $(MKDOCS_BIN) serve

docs-deploy:
	@echo "Deploying documentation to GitHub Pages"
	@$(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) $(MKDOCS_BIN) gh-deploy --strict

docs-check:
	@echo "Checking documentation build integrity"
	@$(DOCS_ENV) ENABLE_SOCIAL_CARDS=$(ENABLE_SOCIAL_CARDS) $(MKDOCS_BIN) build --strict --quiet
	@echo "Documentation passes build checks"

docs-clean:
	@echo "Cleaning documentation build artifacts"
	@rm -rf "$(DOCS_SITE_DIR)"

##@ Documentation
docs: ## Build documentation (mkdocs --strict)
docs-serve: ## Serve documentation locally (auto-reload)
docs-deploy: ## Deploy documentation to GitHub Pages (strict)
docs-check: ## Validate documentation builds without errors
docs-clean: ## Remove generated documentation artifacts
