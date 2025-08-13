# SBOM Configuration

PACKAGE_NAME        ?= bijux-cli
GIT_SHA             ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)
PYPROJECT_VERSION    = $(call read_pyproject_version)
PKG_VERSION         ?= $(PYPROJECT_VERSION)
SBOM_DIR            ?= artifacts/sbom
SBOM_IGNORE         ?= PYSEC-2022-42969
SBOM_PROD_REQ       ?= requirements/prod.txt
SBOM_DEV_REQ        ?= requirements/dev.txt
SBOM_CLI            ?= cyclonedx
SBOM_FORMAT         ?= cyclonedx-json

PIP_AUDIT           := $(ACT)/pip-audit

.PHONY: sbom sbom-prod sbom-dev sbom-validate sbom-clean

sbom: sbom-clean sbom-prod sbom-dev
	@echo "✔ SBOMs generated in $(SBOM_DIR)"

sbom-prod:
	@mkdir -p "$(SBOM_DIR)"
	@if [ -s "$(SBOM_PROD_REQ)" ]; then \
	  echo "→ SBOM (prod via $(SBOM_PROD_REQ))"; \
	  $(PIP_AUDIT) --progress-spinner off --format $(SBOM_FORMAT) \
	    -r "$(SBOM_PROD_REQ)" \
	    --output "$(SBOM_DIR)/$(PACKAGE_NAME)-$(PKG_VERSION)-$(GIT_SHA).prod.cdx.json" \
	    $(foreach V,$(SBOM_IGNORE),--ignore-vuln $(V)) || true; \
	else \
	  echo "→ SBOM (prod fallback: current venv)"; \
	  $(PIP_AUDIT) --progress-spinner off --format $(SBOM_FORMAT) \
	    --output "$(SBOM_DIR)/$(PACKAGE_NAME)-$(PKG_VERSION)-$(GIT_SHA).prod.cdx.json" \
	    $(foreach V,$(SBOM_IGNORE),--ignore-vuln $(V)) || true; \
	fi

sbom-dev:
	@mkdir -p "$(SBOM_DIR)"
	@if [ -s "$(SBOM_DEV_REQ)" ]; then \
	  echo "→ SBOM (dev via $(SBOM_DEV_REQ))"; \
	  $(PIP_AUDIT) --progress-spinner off --format $(SBOM_FORMAT) \
	    -r "$(SBOM_DEV_REQ)" \
	    --output "$(SBOM_DIR)/$(PACKAGE_NAME)-$(PKG_VERSION)-$(GIT_SHA).dev.cdx.json" \
	    $(foreach V,$(SBOM_IGNORE),--ignore-vuln $(V)) || true; \
	else \
	  echo "→ SBOM (dev fallback: current venv)"; \
	  $(PIP_AUDIT) --progress-spinner off --format $(SBOM_FORMAT) \
	    --output "$(SBOM_DIR)/$(PACKAGE_NAME)-$(PKG_VERSION)-$(GIT_SHA).dev.cdx.json" \
	    $(foreach V,$(SBOM_IGNORE),--ignore-vuln $(V)) || true; \
	fi

sbom-validate:
	@if [ -z "$(SBOM_CLI)" ]; then echo "✘ SBOM_CLI not set"; exit 1; fi
	@command -v $(SBOM_CLI) >/dev/null 2>&1 || { echo "✘ '$(SBOM_CLI)' not found. Install it or set SBOM_CLI."; exit 1; }
	@if ! compgen -G "$(SBOM_DIR)/*.cdx.json" >/dev/null; then \
	  echo "✘ No SBOM files in $(SBOM_DIR)"; exit 1; \
	fi
	@for f in "$(SBOM_DIR)"/*.cdx.json; do \
	  echo "→ Validating $$f"; \
	  $(SBOM_CLI) validate --input-format json --input-file "$$f"; \
	done

sbom-clean:
	@echo "→ Cleaning SBOM artifacts"
	@mkdir -p "$(SBOM_DIR)"
	@rm -f $(SBOM_DIR)/$(PACKAGE_NAME)-0.0.0-*.cdx.json \
	       $(SBOM_DIR)/$(PACKAGE_NAME)--*.cdx.json || true

##@ SBOM
sbom: ## Generate SBOMs for prod/dev (pip-audit → CycloneDX JSON)
sbom-validate: ## Validate all generated SBOMs with CycloneDX CLI
sbom-clean: ## Remove all SBOM artifacts from $(SBOM_DIR)
