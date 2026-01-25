# Quality Configuration (evidence → artifacts_pages/quality)

INTERROGATE_PATHS ?= src/bijux_cli
QUALITY_PATHS     ?= src/bijux_cli

VULTURE     := $(ACT)/vulture
DEPTRY      := $(ACT)/deptry
REUSE       := $(ACT)/reuse
INTERROGATE := $(ACT)/interrogate
PYTHON      := $(shell command -v python3 || command -v python)

QUALITY_ARTIFACTS_DIR ?= artifacts/quality
QUALITY_OK_MARKER     := $(QUALITY_ARTIFACTS_DIR)/_passed

ifeq ($(shell uname -s),Darwin)
  BREW_PREFIX  := $(shell command -v brew >/dev/null 2>&1 && brew --prefix)
  CAIRO_PREFIX := $(shell test -n "$(BREW_PREFIX)" && brew --prefix cairo)
  QUALITY_ENV  := DYLD_FALLBACK_LIBRARY_PATH="$(BREW_PREFIX)/lib:$(CAIRO_PREFIX)/lib:$$DYLD_FALLBACK_LIBRARY_PATH"
else
  QUALITY_ENV  :=
endif

.PHONY: quality interrogate-report quality-clean

quality:
	@echo "→ Running quality checks..."
	@mkdir -p "$(QUALITY_ARTIFACTS_DIR)"

	@echo "   - Dead code analysis (Vulture)"
	@set -euo pipefail; \
	  { $(VULTURE) --version 2>/dev/null || echo vulture; } >"$(QUALITY_ARTIFACTS_DIR)/vulture.log"; \
	  OUT="$$( $(VULTURE) $(QUALITY_PATHS) --min-confidence 80 2>&1 || true )"; \
	  printf '%s\n' "$$OUT" >>"$(QUALITY_ARTIFACTS_DIR)/vulture.log"; \
	  if [ -z "$$OUT" ]; then echo "✔ Vulture: no dead code found." >>"$(QUALITY_ARTIFACTS_DIR)/vulture.log"; fi

	@echo "   - Dependency hygiene (Deptry)"
	@set -euo pipefail; \
	  { $(DEPTRY) --version 2>/dev/null || true; } >"$(QUALITY_ARTIFACTS_DIR)/deptry.log"; \
	  $(DEPTRY) $(QUALITY_PATHS) 2>&1 | tee -a "$(QUALITY_ARTIFACTS_DIR)/deptry.log"

	@echo "   - License & SPDX compliance (REUSE)"
	@set -euo pipefail; \
	  { $(REUSE) --version 2>/dev/null || true; } >"$(QUALITY_ARTIFACTS_DIR)/reuse.log"; \
	  $(REUSE) lint 2>&1 | tee -a "$(QUALITY_ARTIFACTS_DIR)/reuse.log"

	@echo "   - Documentation coverage (Interrogate)"
	@$(MAKE) interrogate-report

	@echo "   - E2E contract checks"
	@$(PYTHON) scripts/check_e2e_contract.py

	@echo "✔ Quality checks passed"
	@printf "OK\n" >"$(QUALITY_OK_MARKER)"

interrogate-report:
	@echo "→ Generating docstring coverage report (<100%)"
	@mkdir -p "$(QUALITY_ARTIFACTS_DIR)"
	@set +e; \
	  OUT="$$( $(QUALITY_ENV) $(INTERROGATE) -v $(INTERROGATE_PATHS) )"; \
	  rc=$$?; \
	  printf '%s\n' "$$OUT" >"$(QUALITY_ARTIFACTS_DIR)/interrogate.full.txt"; \
	  OFF="$$(printf '%s\n' "$$OUT" | awk -F'|' 'NR>3 && $$0 ~ /^\|/ { \
	    name=$$2; cov=$$6; gsub(/^[ \t]+|[ \t]+$$/, "", name); gsub(/^[ \t]+|[ \t]+$$/, "", cov); \
	    if (name !~ /^-+$$/ && cov != "100%") printf("  - %s (%s)\n", name, cov); \
	  }')"; \
	  printf '%s\n' "$$OFF" >"$(QUALITY_ARTIFACTS_DIR)/interrogate.offenders.txt"; \
	  if [ -n "$$OFF" ]; then printf '%s\n' "$$OFF"; else echo "✔ All files 100% documented"; fi; \
	  exit $$rc

quality-clean:
	@echo "→ Cleaning quality artifacts"
	@rm -rf "$(QUALITY_ARTIFACTS_DIR)"

##@ Quality
quality: ## Run Vulture, Deptry, REUSE, Interrogate; save logs to artifacts_pages/quality/
interrogate-report: ## Save full Interrogate table + offenders list
quality-clean: ## Remove artifacts_pages/quality
