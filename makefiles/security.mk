# Security Configuration

SECURITY_PATHS   ?= src/bijux_cli
BANDIT           := $(ACT)/bandit
PIP_AUDIT        := $(ACT)/pip-audit
SBOM_IGNORE      ?= PYSEC-2022-42969

.PHONY: security

security:
	@echo "→ Bandit (Python static analysis)" && $(BANDIT) -r $(SECURITY_PATHS)
	@echo "→ Pip-audit (dependency vulnerability scan)" && \
	  $(PIP_AUDIT) $(foreach V,$(SBOM_IGNORE),--ignore-vuln $(V))

##@ Security
security: ## Run Bandit and pip-audit with optional vulnerability ignores
