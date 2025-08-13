# Quality Configuration

QUALITY_PATHS ?= src/bijux_cli
VULTURE       := $(ACT)/vulture
DEPTRY        := $(ACT)/deptry
REUSE         := $(ACT)/reuse
INTERROGATE   := $(ACT)/interrogate

.PHONY: quality

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

##@ Quality
quality: ## Run all quality checks: Vulture (dead code), Deptry (deps), REUSE (SPDX), Interrogate (docs)
