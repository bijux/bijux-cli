# Test Configuration
PYTEST          := $(ACT)/pytest
TEST_PATHS      ?= tests
TEST_PATHS_UNIT ?= tests/unit

.PHONY: test test-unit

test:
	@echo "→ Running full test suite on $(TEST_PATHS)"
	@$(PYTEST) $(TEST_PATHS)
	@$(RM) .coverage* || true

test-unit:
	@echo "→ Running unit tests only"
	@if [ -d "$(TEST_PATHS_UNIT)" ] && find "$(TEST_PATHS_UNIT)" -type f -name 'test_*.py' | grep -q .; then \
	  echo "   • detected $(TEST_PATHS_UNIT) — targeting that directory"; \
	  $(PYTEST) $(TEST_PATHS_UNIT) -m "not slow" --maxfail=1 -q; \
	else \
	  echo "   • no $(TEST_PATHS_UNIT); falling back to exclude e2e/integration/functional/slow"; \
	  $(PYTEST) $(TEST_PATHS) -k "not e2e and not integration and not functional" -m "not slow" --maxfail=1 -q; \
	fi
	@$(RM) .coverage* || true

##@ Test
test: ## Run full test suite with pytest and clean coverage artifacts
test-unit: ## Run unit tests only (prefer tests/unit/, otherwise exclude e2e/integration/functional/slow)
