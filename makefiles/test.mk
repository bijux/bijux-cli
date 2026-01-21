# Test Configuration — zero root pollution (pytest runs from artifacts_pages/test)

TEST_PATHS            ?= tests
TEST_PATHS_UNIT       ?= tests/unit
TEST_PATHS_E2E        ?= tests/e2e
TEST_PATHS_NIGHT      ?= tests/night

TEST_ARTIFACTS_DIR    ?= artifacts/test
JUNIT_XML             ?= $(TEST_ARTIFACTS_DIR)/junit.xml
TMP_DIR               ?= $(TEST_ARTIFACTS_DIR)/tmp
HYPOTHESIS_DB_DIR     ?= $(TEST_ARTIFACTS_DIR)/hypothesis
BENCHMARK_DIR         ?= $(TEST_ARTIFACTS_DIR)/benchmarks

ENABLE_BENCH          ?= 1
PYTEST_ADDOPTS_EXTRA  ?=

# Use the current environment's interpreter/pytest (works under tox/venv/system)
PY 					 ?= python
PYTEST_BIN           := $(shell command -v pytest 2>/dev/null)
PYTEST 				 ?= $(if $(PYTEST_BIN),$(PYTEST_BIN),$(PY) -m pytest)

# absolute paths so running from artifacts_pages/test works cleanly
PYTEST_INI_ABS        := $(abspath pytest.ini)
COVCFG_ABS            := $(abspath config/coveragerc.ini)
COV_HTML_ABS          := $(abspath $(TEST_ARTIFACTS_DIR)/htmlcov)
CACHE_DIR_ABS         := $(abspath $(TEST_ARTIFACTS_DIR)/.pytest_cache)
COV_XML_ABS           := $(abspath $(TEST_ARTIFACTS_DIR)/coverage.xml)

TEST_PATHS_ABS        := $(abspath $(TEST_PATHS))
TEST_PATHS_UNIT_ABS   := $(abspath $(TEST_PATHS_UNIT))
TEST_PATHS_E2E_ABS    := $(abspath $(TEST_PATHS_E2E))
TEST_PATHS_NIGHT_ABS  := $(abspath $(TEST_PATHS_NIGHT))
SRC_ABS               := $(abspath src)
JUNIT_XML_ABS         := $(abspath $(JUNIT_XML))
TMP_DIR_ABS           := $(abspath $(TMP_DIR))
HYPOTHESIS_DB_ABS     := $(abspath $(HYPOTHESIS_DB_DIR))
BENCHMARK_DIR_ABS     := $(abspath $(BENCHMARK_DIR))

# override ini-relative bits with absolute paths
PYTEST_FLAGS = \
  --junitxml "$(JUNIT_XML_ABS)" \
  --basetemp "$(TMP_DIR_ABS)" \
  --cov-config "$(COVCFG_ABS)" \
  --cov-report=html:"$(COV_HTML_ABS)" \
  --cov-report=xml:"$(COV_XML_ABS)" \
  -o cache_dir="$(CACHE_DIR_ABS)" \
  $(PYTEST_ADDOPTS_EXTRA)

PYTEST_FLAGS_NOCOV = \
  --junitxml "$(JUNIT_XML_ABS)" \
  --basetemp "$(TMP_DIR_ABS)" \
  -o cache_dir="$(CACHE_DIR_ABS)" \
  $(PYTEST_ADDOPTS_EXTRA)

.PHONY: test test-unit test-e2e test-night test-clean

test:
	@echo "→ Running full test suite on $(TEST_PATHS)"
	@mkdir -p "$(TEST_ARTIFACTS_DIR)" "$(HYPOTHESIS_DB_DIR)" "$(BENCHMARK_DIR)" "$(TMP_DIR)"
	@rm -rf .hypothesis .benchmarks || true
	@echo "   • JUnit XML → $(JUNIT_XML_ABS)"
	@echo "   • Hypothesis DB → $(HYPOTHESIS_DB_ABS)"
	@echo "   • Using pytest → $(PYTEST)"
	@BENCH_FLAGS=""; \
	if [ "$(ENABLE_BENCH)" = "1" ] && sh -c "$(PYTEST) -q --help" 2>/dev/null | grep -q -- '--benchmark-storage'; then \
	  BENCH_FLAGS="--benchmark-autosave --benchmark-storage=file://$(BENCHMARK_DIR_ABS)"; \
	  echo "   • pytest-benchmark detected → storing in $(BENCHMARK_DIR_ABS)"; \
	else \
	  echo "   • pytest-benchmark disabled or not installed"; \
	fi; \
	( cd "$(TEST_ARTIFACTS_DIR)" && \
	  PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	  HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	  sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_ABS)" $(PYTEST_FLAGS) '"$$BENCH_FLAGS" )
	@rm -rf .hypothesis .benchmarks || true

test-unit:
	@echo "→ Running unit tests only"
	@$(PYTEST) --version
	@echo "pytest cmd: $(PYTEST) -c '$(PYTEST_INI_ABS)' …"
	@mkdir -p "$(TEST_ARTIFACTS_DIR)" "$(HYPOTHESIS_DB_DIR)" "$(BENCHMARK_DIR)" "$(TMP_DIR)"
	@rm -rf .hypothesis .benchmarks || true
	@echo "   • JUnit XML → $(JUNIT_XML_ABS)"
	@echo "   • Hypothesis DB → $(HYPOTHESIS_DB_ABS)"
	@echo "   • Using pytest → $(PYTEST)"
	@BENCH_FLAGS=""; \
	if [ "$(ENABLE_BENCH)" = "1" ] && sh -c "$(PYTEST) -q --help" 2>/dev/null | grep -q -- '--benchmark-storage'; then \
	  BENCH_FLAGS="--benchmark-autosave --benchmark-storage=file://$(BENCHMARK_DIR_ABS)"; \
	  echo "   • pytest-benchmark detected → storing in $(BENCHMARK_DIR_ABS)"; \
	else \
	  echo "   • pytest-benchmark disabled or not installed"; \
	fi; \
	if [ -d "$(TEST_PATHS_UNIT)" ] && find "$(TEST_PATHS_UNIT)" -type f -name 'test_*.py' | grep -q .; then \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" -o addopts= "$(TEST_PATHS_UNIT_ABS)" -m "unit and not slow" --maxfail=1 -q -p no:cov $(PYTEST_FLAGS_NOCOV) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_UNIT); excluding e2e/integration/functional/slow"; \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" -o addopts= "$(TEST_PATHS_ABS)" -k "not e2e and not integration and not functional" -m "not slow" --maxfail=1 -q -p no:cov $(PYTEST_FLAGS_NOCOV) '"$$BENCH_FLAGS" ); \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-e2e:
	@echo "→ Running e2e tests only"
	@$(PYTEST) --version
	@mkdir -p "$(TEST_ARTIFACTS_DIR)" "$(HYPOTHESIS_DB_DIR)" "$(BENCHMARK_DIR)" "$(TMP_DIR)"
	@rm -rf .hypothesis .benchmarks || true
	@echo "   • JUnit XML → $(JUNIT_XML_ABS)"
	@echo "   • Hypothesis DB → $(HYPOTHESIS_DB_ABS)"
	@echo "   • Using pytest → $(PYTEST)"
	@BENCH_FLAGS=""; \
	if [ "$(ENABLE_BENCH)" = "1" ] && sh -c "$(PYTEST) -q --help" 2>/dev/null | grep -q -- '--benchmark-storage'; then \
	  BENCH_FLAGS="--benchmark-autosave --benchmark-storage=file://$(BENCHMARK_DIR_ABS)"; \
	  echo "   • pytest-benchmark detected → storing in $(BENCHMARK_DIR_ABS)"; \
	else \
	  echo "   • pytest-benchmark disabled or not installed"; \
	fi; \
	if [ -d "$(TEST_PATHS_E2E)" ] && find "$(TEST_PATHS_E2E)" -type f -name 'test_*.py' | grep -q .; then \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_E2E_ABS)" -m "e2e and not slow" -q $(PYTEST_FLAGS) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_E2E); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-night:
	@echo "→ Running night tests only"
	@$(PYTEST) --version
	@mkdir -p "$(TEST_ARTIFACTS_DIR)" "$(HYPOTHESIS_DB_DIR)" "$(BENCHMARK_DIR)" "$(TMP_DIR)"
	@rm -rf .hypothesis .benchmarks || true
	@echo "   • JUnit XML → $(JUNIT_XML_ABS)"
	@echo "   • Hypothesis DB → $(HYPOTHESIS_DB_ABS)"
	@echo "   • Using pytest → $(PYTEST)"
	@BENCH_FLAGS=""; \
	if [ "$(ENABLE_BENCH)" = "1" ] && sh -c "$(PYTEST) -q --help" 2>/dev/null | grep -q -- '--benchmark-storage'; then \
	  BENCH_FLAGS="--benchmark-autosave --benchmark-storage=file://$(BENCHMARK_DIR_ABS)"; \
	  echo "   • pytest-benchmark detected → storing in $(BENCHMARK_DIR_ABS)"; \
	else \
	  echo "   • pytest-benchmark disabled or not installed"; \
	fi; \
	if [ -d "$(TEST_PATHS_NIGHT)" ] && find "$(TEST_PATHS_NIGHT)" -type f -name 'test_*.py' | grep -q .; then \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_NIGHT_ABS)" -m "night" -q $(PYTEST_FLAGS) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_NIGHT); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-clean:
	@echo "→ Cleaning test artifacts"
	@rm -rf ".hypothesis" ".benchmarks" || true
	@$(RM) .coverage* || true
	@echo "✔ done"

##@ Test
test: ## Run full test suite; all side-effects contained in artifacts_pages/test/ (JUnit, htmlcov, tmp, hypothesis DB, benchmarks)
test-unit: ## Run unit tests only; same containment; fallback excludes e2e/integration/functional/slow
test-e2e: ## Run e2e tests only (marked), not recommended for quick runs
test-night: ## Run night tests only (marked), excluded from default runs
test-clean: ## Remove stray root .hypothesis/.benchmarks and coverage files
