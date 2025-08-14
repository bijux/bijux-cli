# Quality Configuration

INTERROGATE_PATHS ?= src/bijux_cli
QUALITY_PATHS     ?= src/bijux_cli

VULTURE     := $(ACT)/vulture
DEPTRY      := $(ACT)/deptry
REUSE       := $(ACT)/reuse
INTERROGATE := $(ACT)/interrogate
PYTHON      := $(shell command -v python3 || command -v python)

ifeq ($(shell uname -s),Darwin)
  BREW_PREFIX := $(shell command -v brew >/dev/null 2>&1 && brew --prefix)
  CAIRO_PREFIX := $(shell test -n "$(BREW_PREFIX)" && brew --prefix cairo)
  QUALITY_ENV := DYLD_FALLBACK_LIBRARY_PATH="$(BREW_PREFIX)/lib:$(CAIRO_PREFIX)/lib:$$DYLD_FALLBACK_LIBRARY_PATH"
else
  QUALITY_ENV :=
endif

.PHONY: quality interrogate-report

quality:
	@echo "→ Running quality checks..."
	@echo "   - Dead code analysis (Vulture)"
	@$(VULTURE) $(QUALITY_PATHS) --min-confidence 80
	@echo "   - Dependency hygiene (Deptry)"
	@$(DEPTRY) $(QUALITY_PATHS)
	@echo "   - License & SPDX compliance (REUSE)"
	@$(REUSE) lint
	@echo "   - Documentation coverage (Interrogate)"
	@$(MAKE) interrogate-report
	@echo "✔ Quality checks passed"

interrogate-report:
	@echo "→ Generating docstring coverage report (<100%)"
	@set +e; \
      OUT="$$( $(QUALITY_ENV) $(INTERROGATE) --verbose $(INTERROGATE_PATHS) )"; \
      rc=$$?; \
      OFF="$$(printf '%s\n' "$$OUT" | awk -F'|' 'NR>3 && $$0 ~ /^\|/ { \
        name=$$2; cov=$$6; gsub(/^[ \t]+|[ \t]+$$/, "", name); gsub(/^[ \t]+|[ \t]+$$/, "", cov); \
        if (name !~ /^-+$$/ && cov != "100%") printf("  - %s (%s)\n", name, cov); \
      }')"; \
      if [ -n "$$OFF" ]; then printf "%s\n" "$$OFF"; else echo "✔ All files 100% documented"; fi; \
      exit $$rc

##@ Quality
quality: ## Run all quality checks: Vulture (dead code), Deptry (deps), REUSE (SPDX), Interrogate (docs)
interrogate-report: ## Generate docstring coverage report for files <100%
