# Documentation Configuration

MKDOCS        := $(ACT)/mkdocs
DOCS_SITE_DIR := site

.PHONY: docs docs-clean docs-serve docs-deploy docs-check

docs: docs-clean
	@echo "→ Building documentation"
	@$(MKDOCS) build --strict
	@echo "✔ Documentation build complete"

docs-serve:
	@echo "→ Serving documentation on localhost"
	@$(MKDOCS) serve

docs-deploy:
	@echo "→ Deploying documentation to GitHub Pages"
	@$(MKDOCS) gh-deploy --strict

docs-check:
	@echo "→ Checking documentation build integrity"
	@$(MKDOCS) build --strict --quiet
	@echo "✔ Documentation passes build checks"

docs-clean:
	@echo "→ Cleaning documentation build artifacts"
	@rm -rf "$(DOCS_SITE_DIR)"

##@ Documentation
docs: ## Build documentation (mkdocs --strict)
docs-serve: ## Serve documentation locally (auto-reload)
docs-deploy: ## Deploy documentation to GitHub Pages (strict)
docs-check: ## Validate documentation builds without errors
docs-clean: ## Remove generated documentation artifacts
