# Test Configuration — zero root pollution (pytest runs from artifacts_pages/test)

TEST_PATHS            ?= tests
TEST_PATHS_UNIT       ?= tests/unit
TEST_PATHS_E2E        ?= tests/e2e
TEST_PATHS_NIGHTLY    ?= tests/nightly
TEST_PATHS_REGRESSION ?= tests/regression
TEST_PATHS_BENCHMARK  ?= tests/benchmark

TEST_ARTIFACTS_DIR    ?= artifacts/test
JUNIT_XML             ?= $(TEST_ARTIFACTS_DIR)/junit.xml
JUNIT_XML_UNIT        ?= $(TEST_ARTIFACTS_DIR)/junit-test-unit.xml
JUNIT_XML_E2E         ?= $(TEST_ARTIFACTS_DIR)/junit-test-e2e.xml
JUNIT_XML_NIGHTLY     ?= $(TEST_ARTIFACTS_DIR)/junit-test-nightly.xml
JUNIT_XML_REGRESSION  ?= $(TEST_ARTIFACTS_DIR)/junit-test-regression.xml
JUNIT_XML_BENCHMARK   ?= $(TEST_ARTIFACTS_DIR)/junit-test-benchmark.xml
TMP_DIR               ?= $(TEST_ARTIFACTS_DIR)/tmp
HYPOTHESIS_DB_DIR     ?= $(TEST_ARTIFACTS_DIR)/hypothesis
BENCHMARK_DIR         ?= $(TEST_ARTIFACTS_DIR)/benchmarks
BIJUX_BIN             ?= bin/bijux

ENABLE_BENCH          ?= 1
PYTEST_ADDOPTS_EXTRA  ?=

# Use the current environment's interpreter/pytest (works under tox/venv/system)
PY 					 ?= python
PYTEST_BIN           := $(shell command -v pytest 2>/dev/null)
PYTEST 				 ?= $(if $(PYTEST_BIN),$(PYTEST_BIN),$(PY) -m pytest)
PYTHON_311           ?= $(shell command -v python3.11 2>/dev/null || command -v python3 2>/dev/null || command -v python 2>/dev/null)

# absolute paths so running from artifacts_pages/test works cleanly
PYTEST_INI_ABS        := $(abspath pytest.ini)
COVCFG_ABS            := $(abspath configs/coveragerc.ini)
COV_HTML_ABS          := $(abspath $(TEST_ARTIFACTS_DIR)/htmlcov)
CACHE_DIR_ABS         := $(abspath $(TEST_ARTIFACTS_DIR)/.pytest_cache)
COV_XML_ABS           := $(abspath $(TEST_ARTIFACTS_DIR)/coverage.xml)

TEST_PATHS_ABS        := $(abspath $(TEST_PATHS))
TEST_PATHS_UNIT_ABS   := $(abspath $(TEST_PATHS_UNIT))
TEST_PATHS_E2E_ABS    := $(abspath $(TEST_PATHS_E2E))
TEST_PATHS_NIGHTLY_ABS := $(abspath $(TEST_PATHS_NIGHTLY))
TEST_PATHS_REGRESSION_ABS := $(abspath $(TEST_PATHS_REGRESSION))
TEST_PATHS_BENCHMARK_ABS := $(abspath $(TEST_PATHS_BENCHMARK))
SRC_ABS               := $(abspath src)
JUNIT_XML_ABS         = $(abspath $(JUNIT_XML))
TMP_DIR_ABS           := $(abspath $(TMP_DIR))
HYPOTHESIS_DB_ABS     := $(abspath $(HYPOTHESIS_DB_DIR))
BENCHMARK_DIR_ABS     := $(abspath $(BENCHMARK_DIR))
BIJUX_BIN_ABS         := $(abspath $(BIJUX_BIN))

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


.PHONY: test test-all test-unit test-e2e test-nightly test-regression test-benchmark test-clean

test:
	@echo "→ Running full test suite on $(TEST_PATHS)"
	@mkdir -p "$(TEST_ARTIFACTS_DIR)" "$(HYPOTHESIS_DB_DIR)" "$(BENCHMARK_DIR)" "$(TMP_DIR)"
	@rm -rf .hypothesis .benchmarks || true

test-all:
	@echo "→ Running full test suite (including slow/nightly/bench)"
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
	  BIJUX_NIGHTLY=1 sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_ABS)" -o addopts= -o timeout=60 $(PYTEST_FLAGS) '"$$BENCH_FLAGS" )
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

test-unit: JUNIT_XML=$(JUNIT_XML_UNIT)
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
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_UNIT_ABS)" -m "unit and not slow" --maxfail=1 -q $(PYTEST_FLAGS) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_UNIT); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-e2e: JUNIT_XML=$(JUNIT_XML_E2E)
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
	    BIJUX_NIGHTLY=0 sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_E2E_ABS)" -m "e2e" -q -o addopts= -o timeout=10 $(PYTEST_FLAGS) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_E2E); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-nightly: JUNIT_XML=$(JUNIT_XML_NIGHTLY)
test-nightly:
	@echo "→ Running nightly tests only"
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
	if [ -d "$(TEST_PATHS_NIGHTLY)" ] && find "$(TEST_PATHS_NIGHTLY)" -type f -name 'test_*.py' | grep -q .; then \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    BIJUX_NIGHTLY=1 sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_NIGHTLY_ABS)" -m "nightly" -q -o addopts= -o timeout=10 $(PYTEST_FLAGS) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_NIGHTLY); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-regression: JUNIT_XML=$(JUNIT_XML_REGRESSION)
test-regression:
	@echo "→ Running regression tests (functional + integration)"
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
	if [ -d "$(TEST_PATHS_REGRESSION)" ] && find "$(TEST_PATHS_REGRESSION)" -type f -name 'test_*.py' | grep -q .; then \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    BIJUX_BIN="$(BIJUX_BIN_ABS)" \
	    BIJUXCLI_PLUGINS_DIR="$(TMP_DIR_ABS)/plugins" \
	    BIJUX_PYTHON="$(PYTHON_311)" \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_REGRESSION_ABS)" -m "not nightly and not slow" -q -o addopts= $(PYTEST_FLAGS_NOCOV) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_REGRESSION); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-benchmark: JUNIT_XML=$(JUNIT_XML_BENCHMARK)
test-benchmark:
	@echo "→ Running benchmark tests only"
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
	if [ -d "$(TEST_PATHS_BENCHMARK)" ] && find "$(TEST_PATHS_BENCHMARK)" -type f -name 'test_*.py' | grep -q .; then \
	  ( cd "$(TEST_ARTIFACTS_DIR)" && \
	    PYTHONPATH="$(SRC_ABS)$${PYTHONPATH:+:$${PYTHONPATH}}" \
	    HYPOTHESIS_DATABASE_DIRECTORY="$(HYPOTHESIS_DB_ABS)" \
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_BENCHMARK_ABS)" -q -o addopts= $(PYTEST_FLAGS_NOCOV) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_BENCHMARK); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-clean:
	@echo "→ Cleaning test artifacts"
	@rm -rf ".hypothesis" ".benchmarks" || true
	@$(RM) .coverage* || true
	@echo "✔ done"

##@ Test
test: ## Run full test suite; all side-effects contained in artifacts_pages/test/ (JUnit, htmlcov, tmp, hypothesis DB, benchmarks)
test-all: ## Run all tests including slow, nightly, and benchmarks
test-unit: ## Run unit tests only; same containment; fallback excludes e2e/integration/functional/slow
test-e2e: ## Run e2e tests only (marked), not recommended for quick runs
test-nightly: ## Run nightly tests only (marked), excluded from default runs
test-regression: ## Run functional + integration tests
test-clean: ## Remove stray root .hypothesis/.benchmarks and coverage files
