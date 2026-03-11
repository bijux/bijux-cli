# Security Configuration (no SBOM here; SBOM is handled in sbom.mk)

SECURITY_PATHS           ?= crates/bijux-cli-python/python/bijux_cli_py
BANDIT                   ?= $(if $(ACT),$(ACT)/bandit,bandit)
PIP_AUDIT                ?= $(if $(ACT),$(ACT)/pip-audit,pip-audit)
VENV_PYTHON              ?= $(if $(VIRTUAL_ENV),$(VIRTUAL_ENV)/bin/python,python)

SECURITY_REPORT_DIR      ?= artifacts/security
BANDIT_JSON              := $(SECURITY_REPORT_DIR)/bandit.json
BANDIT_TXT               := $(SECURITY_REPORT_DIR)/bandit.txt
PIPA_JSON                := $(SECURITY_REPORT_DIR)/pip-audit.json
PIPA_TXT                 := $(SECURITY_REPORT_DIR)/pip-audit.txt

SECURITY_IGNORE_IDS      ?=
SECURITY_IGNORE_FLAGS     = $(foreach V,$(SECURITY_IGNORE_IDS),--ignore-vuln $(V))
PIP_AUDIT_CONSOLE_FLAGS  ?= --skip-editable --progress-spinner off
PIP_AUDIT_INPUTS         ?=
SECURITY_STRICT          ?= 1

BANDIT_EXCLUDES          ?= .venv,venv,build,dist,.tox,.mypy_cache,.pytest_cache
BANDIT_THREADS           ?= 0

.PHONY: security security-bandit security-audit security-clean

security: security-bandit security-audit

security-bandit:
	@mkdir -p "$(SECURITY_REPORT_DIR)"
	@echo "→ Bandit (Python static analysis)"
	@$(BANDIT) -r "$(SECURITY_PATHS)" -x "$(BANDIT_EXCLUDES)" -f json -o "$(BANDIT_JSON)" -n $(BANDIT_THREADS) || true
	@$(BANDIT) -r "$(SECURITY_PATHS)" -x "$(BANDIT_EXCLUDES)" -n $(BANDIT_THREADS) | tee "$(BANDIT_TXT)"

security-audit:
	@mkdir -p "$(SECURITY_REPORT_DIR)"
	@echo "→ Pip-audit (dependency vulnerability scan)"
	@$(PIP_AUDIT) $(SECURITY_IGNORE_FLAGS) $(PIP_AUDIT_CONSOLE_FLAGS) $(PIP_AUDIT_INPUTS) \
	  -f json -o "$(PIPA_JSON)" >/dev/null 2>&1 || \
	  echo "!  pip-audit invocation failed (rc=$$?)"
	@set -o pipefail; \
	PIPA_JSON="$(PIPA_JSON)" \
	SECURITY_STRICT="$(SECURITY_STRICT)" \
	SECURITY_IGNORE_IDS="$(SECURITY_IGNORE_IDS)" \
	"$(VENV_PYTHON)" scripts/helper_pip_audit.py | tee "$(PIPA_TXT)"

security-clean:
	@rm -rf "$(SECURITY_REPORT_DIR)"

##@ Security
security:        ## Run Bandit and pip-audit; save reports to $(SECURITY_REPORT_DIR)
security-bandit: ## Run Bandit (screen + JSON artifact)
security-audit:  ## Run pip-audit (JSON once) and gate via scripts/helper_pip_audit.py; prints concise summary
security-clean:  ## Remove security reports
