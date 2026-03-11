# Python quality, packaging, and release lane (single ownership module)
# Covers lint, test, quality, security, build, sbom, and publish workflows.

PYTHON_CONFIG_DIR ?= configs/python

# -------------------------------
# Python lint
# -------------------------------
RUFF        := $(ACT)/ruff
MYPY        := $(ACT)/mypy
CODESPELL   := $(ACT)/codespell
PYDOCSTYLE  := $(ACT)/pydocstyle
RADON       := $(ACT)/radon

LINT_DIRS           ?= crates/bijux-cli-python/python/bijux_cli_py crates/bijux-cli-python/tests/python
LINT_ARTIFACTS_DIR  ?= artifacts/lint
RUFF_CACHE_DIR      ?= $(LINT_ARTIFACTS_DIR)/.ruff_cache
MYPY_CACHE_DIR      ?= $(LINT_ARTIFACTS_DIR)/.mypy_cache
VENV_PYTHON         ?= python3

.PHONY: \
	fmt-py fmt-check-py lint-py lint-artifacts-py lint-file-py lint-dir-py lint-clean-py \
	fmt fmt-check lint lint-artifacts lint-file lint-dir lint-clean

fmt-py: | $(VENV)
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -euo pipefail; { \
	  echo "→ Ruff format"; \
	  $(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS); \
	} 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"

fmt-check-py: | $(VENV)
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -euo pipefail; { \
	  echo "→ Ruff format (check)"; \
	  $(RUFF) format --check --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS); \
	} 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"

lint-py: lint-artifacts-py
	@echo "✔ Python linting completed (logs in '$(LINT_ARTIFACTS_DIR)')"

lint-artifacts-py: | $(VENV)
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)" "$(MYPY_CACHE_DIR)"
	@set -euo pipefail; { \
	  echo "→ Ruff format (check)"; \
	  $(RUFF) format --check --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS); \
	} 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"
	@set -euo pipefail; $(RUFF) check --fix --config "$(PYTHON_CONFIG_DIR)/ruff.toml" --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff.log"
	@set -euo pipefail; $(MYPY) --config-file "$(PYTHON_CONFIG_DIR)/mypy.ini" --strict --cache-dir "$(MYPY_CACHE_DIR)" $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/mypy.log"
	@set -euo pipefail; $(CODESPELL) -I "$(PYTHON_CONFIG_DIR)/bijux.dic" $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/codespell.log"
	@set -euo pipefail; $(RADON) cc -s -a $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/radon.log"
	@set -euo pipefail; $(PYDOCSTYLE) --convention=google $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/pydocstyle.log"
	@[ -d .mypy_cache ] && echo "→ removing stray .mypy_cache" && rm -rf .mypy_cache || true
	@[ -d .ruff_cache ] && echo "→ removing stray .ruff_cache" && rm -rf .ruff_cache || true
	@printf "OK\n" > "$(LINT_ARTIFACTS_DIR)/_passed"

lint-file-py:
ifndef file
	$(error Usage: make lint-file-py file=path/to/file.py)
endif
	@$(call run_tool,RuffFormat,$(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)")
	@$(call run_tool,Ruff,$(RUFF) check --fix --config "$(PYTHON_CONFIG_DIR)/ruff.toml" --cache-dir "$(RUFF_CACHE_DIR)")
	@$(call run_tool,Mypy,$(MYPY) --config-file "$(PYTHON_CONFIG_DIR)/mypy.ini" --strict --cache-dir "$(MYPY_CACHE_DIR)")
	@$(call run_tool,Codespell,$(CODESPELL) -I "$(PYTHON_CONFIG_DIR)/bijux.dic")
	@$(call run_tool,Radon,$(RADON) cc -s -a)
	@$(call run_tool,Pydocstyle,$(PYDOCSTYLE) --convention=google)

lint-dir-py:
ifndef dir
	$(error Usage: make lint-dir-py dir=<directory_path>)
endif
	@$(MAKE) LINT_DIRS="$(dir)" lint-artifacts-py

lint-clean-py:
	@echo "→ Cleaning Python lint artifacts"
	@rm -rf "$(LINT_ARTIFACTS_DIR)" .mypy_cache .ruff_cache || true
	@echo "✔ done"

# Backward-compatible aliases
fmt: fmt-py
fmt-check: fmt-check-py
lint: lint-py
lint-artifacts: lint-artifacts-py
lint-file: lint-file-py
lint-dir: lint-dir-py
lint-clean: lint-clean-py

##@ Python Lint
fmt-py: ## Auto-format Python code using Ruff
fmt-check-py: ## Check Python formatting with Ruff (no changes)
lint-py: ## Run all Python lint checks; save logs to artifacts/lint/
lint-artifacts-py: ## Same as 'lint-py' (explicit), generates logs
lint-file-py: ## Lint a single Python file (requires file=<path>)
lint-dir-py: ## Lint a Python directory (requires dir=<path>)
lint-clean-py: ## Remove Python lint artifacts and caches

# -------------------------------
# Python tests
# -------------------------------
TEST_PATHS            ?= crates/bijux-cli-python/tests/python
TEST_PATHS_UNIT       ?= crates/bijux-cli-python/tests/python
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
BIJUX_BIN             ?= $(shell command -v bijux 2>/dev/null || command -v bijux-rs 2>/dev/null || echo artifacts/rust/target/debug/bijux)

ENABLE_BENCH          ?= 1
PYTEST_ADDOPTS_EXTRA  ?=

PY                     ?= python
PYTEST_BIN            := $(shell command -v pytest 2>/dev/null)
PYTEST                ?= $(if $(PYTEST_BIN),$(PYTEST_BIN),$(PY) -m pytest)
PYTHON_311            ?= $(shell command -v python3.11 2>/dev/null || command -v python3 2>/dev/null || command -v python 2>/dev/null)

PYTEST_INI_ABS        := $(abspath $(PYTHON_CONFIG_DIR)/pytest.ini)
COVCFG_ABS            := $(abspath $(PYTHON_CONFIG_DIR)/coveragerc.ini)
COV_HTML_ABS          := $(abspath $(TEST_ARTIFACTS_DIR)/htmlcov)
CACHE_DIR_ABS         := $(abspath $(TEST_ARTIFACTS_DIR)/.pytest_cache)
COV_XML_ABS           := $(abspath $(TEST_ARTIFACTS_DIR)/coverage.xml)

TEST_PATHS_ABS         := $(abspath $(TEST_PATHS))
TEST_PATHS_UNIT_ABS    := $(abspath $(TEST_PATHS_UNIT))
TEST_PATHS_E2E_ABS     := $(abspath $(TEST_PATHS_E2E))
TEST_PATHS_NIGHTLY_ABS := $(abspath $(TEST_PATHS_NIGHTLY))
TEST_PATHS_REGRESSION_ABS := $(abspath $(TEST_PATHS_REGRESSION))
TEST_PATHS_BENCHMARK_ABS  := $(abspath $(TEST_PATHS_BENCHMARK))
SRC_ABS                := $(abspath crates/bijux-cli-python/python)
JUNIT_XML_ABS          = $(abspath $(JUNIT_XML))
TMP_DIR_ABS            := $(abspath $(TMP_DIR))
HYPOTHESIS_DB_ABS      := $(abspath $(HYPOTHESIS_DB_DIR))
BENCHMARK_DIR_ABS      := $(abspath $(BENCHMARK_DIR))
BIJUX_BIN_ABS          := $(abspath $(BIJUX_BIN))

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

.PHONY: \
	test-py test-all-py test-unit-py test-e2e-py test-nightly-py test-regression-py test-benchmark-py test-clean-py \
	test test-all test-unit test-e2e test-nightly test-night test-regression test-benchmark test-clean

test-py:
	@echo "→ Running Python test suite on $(TEST_PATHS)"
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

test-all-py:
	@echo "→ Running full Python test suite (including slow/nightly/bench)"
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

test-unit-py: JUNIT_XML=$(JUNIT_XML_UNIT)
test-unit-py:
	@echo "→ Running Python unit tests only"
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
	    sh -c '$(PYTEST) -c "$(PYTEST_INI_ABS)" "$(TEST_PATHS_UNIT_ABS)" -m "not slow" --maxfail=1 -q $(PYTEST_FLAGS) '"$$BENCH_FLAGS" ); \
	else \
	  echo "   • no $(TEST_PATHS_UNIT); nothing to run"; \
	fi
	@rm -rf .hypothesis .benchmarks || true

test-e2e-py: JUNIT_XML=$(JUNIT_XML_E2E)
test-e2e-py:
	@echo "→ Running Python e2e tests only"
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

test-nightly-py: JUNIT_XML=$(JUNIT_XML_NIGHTLY)
test-nightly-py:
	@echo "→ Running Python nightly tests only"
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

test-regression-py: JUNIT_XML=$(JUNIT_XML_REGRESSION)
test-regression-py:
	@echo "→ Running Python regression tests (functional + integration)"
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

test-benchmark-py: JUNIT_XML=$(JUNIT_XML_BENCHMARK)
test-benchmark-py:
	@echo "→ Running Python benchmark tests only"
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

test-clean-py:
	@echo "→ Cleaning Python test artifacts"
	@rm -rf ".hypothesis" ".benchmarks" || true
	@$(RM) .coverage* || true
	@echo "✔ done"

# Backward-compatible aliases
test: test-py
test-all: test-all-py
test-unit: test-unit-py
test-e2e: test-e2e-py
test-nightly: test-nightly-py
test-night: test-nightly-py
test-regression: test-regression-py
test-benchmark: test-benchmark-py
test-clean: test-clean-py

##@ Python Test
test-py: ## Run default Python test suite (not nightly/slow)
test-all-py: ## Run all Python tests including slow and nightly
test-unit-py: ## Run Python unit tests only
test-e2e-py: ## Run Python e2e tests only
test-nightly-py: ## Run Python nightly tests only
test-regression-py: ## Run Python functional + integration tests
test-benchmark-py: ## Run Python benchmark tests only
test-clean-py: ## Remove Python test artifacts and coverage leftovers

# -------------------------------
# Python quality
# -------------------------------
INTERROGATE_PATHS ?= crates/bijux-cli-python/python/bijux_cli_py
QUALITY_PATHS     ?= crates/bijux-cli-python/python/bijux_cli_py

VULTURE     := $(ACT)/vulture
DEPTRY      := $(ACT)/deptry
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

.PHONY: quality-py interrogate-report-py quality-clean-py quality interrogate-report quality-clean

quality-py:
	@echo "→ Running Python quality checks..."
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

	@echo "   - Documentation coverage (Interrogate)"
	@$(MAKE) interrogate-report-py

	@echo "   - E2E contract checks"
	@$(PYTHON) scripts/check_e2e_contract.py

	@echo "✔ Python quality checks passed"
	@printf "OK\n" >"$(QUALITY_OK_MARKER)"

interrogate-report-py:
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

quality-clean-py:
	@echo "→ Cleaning Python quality artifacts"
	@rm -rf "$(QUALITY_ARTIFACTS_DIR)"

# Backward-compatible aliases
quality: quality-py
interrogate-report: interrogate-report-py
quality-clean: quality-clean-py

##@ Python Quality
quality-py: ## Run Vulture, Deptry, Interrogate; save logs to artifacts/quality/
interrogate-report-py: ## Save full Interrogate table + offenders list
quality-clean-py: ## Remove artifacts/quality

# -------------------------------
# Python security
# -------------------------------
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

.PHONY: security-py security-bandit-py security-audit-py security-clean-py security security-bandit security-audit security-clean

security-py: security-bandit-py security-audit-py

security-bandit-py:
	@mkdir -p "$(SECURITY_REPORT_DIR)"
	@echo "→ Bandit (Python static analysis)"
	@$(BANDIT) -r "$(SECURITY_PATHS)" -x "$(BANDIT_EXCLUDES)" -f json -o "$(BANDIT_JSON)" -n $(BANDIT_THREADS) || true
	@$(BANDIT) -r "$(SECURITY_PATHS)" -x "$(BANDIT_EXCLUDES)" -n $(BANDIT_THREADS) | tee "$(BANDIT_TXT)"

security-audit-py:
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

security-clean-py:
	@rm -rf "$(SECURITY_REPORT_DIR)"

# Backward-compatible aliases
security: security-py
security-bandit: security-bandit-py
security-audit: security-audit-py
security-clean: security-clean-py

##@ Python Security
security-py:        ## Run Bandit and pip-audit; save reports to artifacts/security
security-bandit-py: ## Run Bandit (screen + JSON artifact)
security-audit-py:  ## Run pip-audit (JSON once) and gate via scripts/helper_pip_audit.py
security-clean-py:  ## Remove Python security reports

# -------------------------------
# Python packaging build
# -------------------------------
BUILD_DIR        ?= artifacts/build
CHECK_DISTS      ?= 1
PYTHON_DIST_DIR  ?= crates/bijux-cli-python

BUILD_DIR_ABS    := $(abspath $(BUILD_DIR))
PYPROJECT_ABS    := $(abspath $(PYTHON_DIST_DIR)/pyproject.toml)

.PHONY: build-py build-sdist-py build-wheel-py build-check-py build-tools-py build-clean-py build build-sdist build-wheel build-check build-tools build-clean

build-tools-py: | $(VENV)
	@echo "→ Ensuring Python build toolchain..."
	@$(VENV_PYTHON) -m pip install -U pip
	@$(VENV_PYTHON) -m pip install --upgrade build twine maturin

build-py: build-tools-py
	@if [ ! -f "$(PYPROJECT_ABS)" ]; then echo "✘ pyproject.toml not found"; exit 1; fi
	@echo "→ Preparing Python package artifacts..."
	@mkdir -p "$(BUILD_DIR_ABS)"
	@echo "→ Building wheel + sdist from $(PYTHON_DIST_DIR) → $(BUILD_DIR_ABS)"
	@$(VENV_PYTHON) -m build --wheel --sdist --outdir "$(BUILD_DIR_ABS)" "$(PYTHON_DIST_DIR)"
	@if [ "$(CHECK_DISTS)" = "1" ]; then \
	  echo "→ Validating distributions with twine"; \
	  $(VENV_PYTHON) -m twine check "$(BUILD_DIR_ABS)"/* 2>&1 | tee "$(BUILD_DIR_ABS)/twine-check.log"; \
	else \
	  echo "→ Skipping twine check (CHECK_DISTS=$(CHECK_DISTS))"; \
	fi
	@echo "✔ Build artifacts ready in '$(BUILD_DIR_ABS)'"
	@ls -l "$(BUILD_DIR_ABS)" || true

build-sdist-py: build-tools-py
	@if [ ! -f "$(PYPROJECT_ABS)" ]; then echo "✘ pyproject.toml not found"; exit 1; fi
	@mkdir -p "$(BUILD_DIR_ABS)"
	@echo "→ Building sdist from $(PYTHON_DIST_DIR) → $(BUILD_DIR_ABS)"
	@$(VENV_PYTHON) -m build --sdist --outdir "$(BUILD_DIR_ABS)" "$(PYTHON_DIST_DIR)"

build-wheel-py: build-tools-py
	@if [ ! -f "$(PYPROJECT_ABS)" ]; then echo "✘ pyproject.toml not found"; exit 1; fi
	@mkdir -p "$(BUILD_DIR_ABS)"
	@echo "→ Building wheel from $(PYTHON_DIST_DIR) → $(BUILD_DIR_ABS)"
	@$(VENV_PYTHON) -m build --wheel --outdir "$(BUILD_DIR_ABS)" "$(PYTHON_DIST_DIR)"

build-check-py:
	@if ls "$(BUILD_DIR_ABS)"/* 1>/dev/null 2>&1; then \
	  $(VENV_PYTHON) -m twine check "$(BUILD_DIR_ABS)"/* 2>&1 | tee "$(BUILD_DIR_ABS)/twine-check.log"; \
	else \
	  echo "✘ No artifacts in $(BUILD_DIR_ABS) to check"; exit 1; \
	fi

build-clean-py:
	@echo "→ Cleaning Python build artifacts..."
	@rm -rf "$(BUILD_DIR_ABS)" || true
	@rm -rf build dist *.egg-info || true
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@echo "✔ Build artifacts cleaned"

# Backward-compatible aliases
build-tools: build-tools-py
build: build-py
build-sdist: build-sdist-py
build-wheel: build-wheel-py
build-check: build-check-py
build-clean: build-clean-py

##@ Python Build
build-tools-py: ## Ensure local venv has Python build tooling (pip, build, twine)
build-clean-py: ## Remove Python build artifacts (artifacts/build + legacy build/, dist/, *.egg-info)
build-py: ## Build wheel and source distribution into artifacts/build
build-sdist-py: ## Build Python sdist only into artifacts/build
build-wheel-py: ## Build Python wheel only into artifacts/build
build-check-py: ## Run twine check on artifacts/build/*

# -------------------------------
# Python SBOM
# -------------------------------
PACKAGE_NAME        ?= bijux-cli
GIT_SHA             ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)

GIT_TAG_EXACT       := $(shell git describe --tags --exact-match 2>/dev/null | sed -E 's/^v//')
GIT_TAG_LATEST      := $(shell git describe --tags --abbrev=0 2>/dev/null | sed -E 's/^v//')

PYPROJECT_VERSION    = $(call read_pyproject_version)

SBOM_PKG_VERSION     := $(strip $(if $(GIT_TAG_EXACT),$(GIT_TAG_EXACT),\
                           $(if $(PYPROJECT_VERSION),$(PYPROJECT_VERSION),\
                           $(if $(GIT_TAG_LATEST),$(GIT_TAG_LATEST),0.0.0))))

GIT_DESCRIBE        := $(shell git describe --tags --long --dirty --always 2>/dev/null)
SBOM_PKG_VERSION_FULL := $(strip $(if $(GIT_TAG_EXACT),$(SBOM_PKG_VERSION),\
                          $(shell echo "$(GIT_DESCRIBE)" \
                            | sed -E 's/^v//; s/-([0-9]+)-g([0-9a-f]+)(-dirty)?$$/+\1.g\2\3/')))

SBOM_VERSION        := $(strip $(if $(SBOM_PKG_VERSION_FULL),$(SBOM_PKG_VERSION_FULL),$(SBOM_PKG_VERSION)))

SBOM_DIR            ?= artifacts/sbom
SBOM_PROD_REQ       ?= requirements/prod.txt
SBOM_DEV_REQ        ?= requirements/dev.txt
SBOM_FORMAT         ?= cyclonedx-json
SBOM_CLI            ?= cyclonedx
SBOM_IGNORE_IDS     ?= PYSEC-2022-42969
SBOM_IGNORE_FLAGS    = $(foreach V,$(SBOM_IGNORE_IDS),--ignore-vuln $(V))

PIP_AUDIT           := $(if $(ACT),$(ACT)/pip-audit,pip-audit)
PIP_AUDIT_FLAGS      = --progress-spinner off --format $(SBOM_FORMAT)

SBOM_PROD_FILE      := $(SBOM_DIR)/$(PACKAGE_NAME)-$(SBOM_VERSION)-$(GIT_SHA).prod.cdx.json
SBOM_DEV_FILE       := $(SBOM_DIR)/$(PACKAGE_NAME)-$(SBOM_VERSION)-$(GIT_SHA).dev.cdx.json

.PHONY: sbom-py sbom-prod-py sbom-dev-py sbom-validate-py sbom-summary-py sbom-clean-py sbom sbom-prod sbom-dev sbom-validate sbom-summary sbom-clean

sbom-py: sbom-clean-py sbom-prod-py sbom-dev-py sbom-summary-py
	@echo "✔ Python SBOMs generated in $(SBOM_DIR)"

sbom-prod-py:
	@mkdir -p "$(SBOM_DIR)"
	@if [ -s "$(SBOM_PROD_REQ)" ]; then \
	  echo "→ SBOM (prod via $(SBOM_PROD_REQ))"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    -r "$(SBOM_PROD_REQ)" --output "$(SBOM_PROD_FILE)" || true; \
	else \
	  echo "→ SBOM (prod fallback: current venv)"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    --output "$(SBOM_PROD_FILE)" || true; \
	fi

sbom-dev-py:
	@mkdir -p "$(SBOM_DIR)"
	@if [ -s "$(SBOM_DEV_REQ)" ]; then \
	  echo "→ SBOM (dev via $(SBOM_DEV_REQ))"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    -r "$(SBOM_DEV_REQ)" --output "$(SBOM_DEV_FILE)" || true; \
	else \
	  echo "→ SBOM (dev fallback: current venv)"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    --output "$(SBOM_DEV_FILE)" || true; \
	fi

sbom-validate-py:
	@if [ -z "$(SBOM_CLI)" ]; then echo "✘ SBOM_CLI not set"; exit 1; fi
	@command -v $(SBOM_CLI) >/dev/null 2>&1 || { echo "✘ '$(SBOM_CLI)' not found. Install it or set SBOM_CLI."; exit 1; }
	@if ! find "$(SBOM_DIR)" -maxdepth 1 -name '*.cdx.json' -print -quit | grep -q .; then \
	  echo "✘ No SBOM files in $(SBOM_DIR)"; exit 1; \
	fi
	@for f in "$(SBOM_DIR)"/*.cdx.json; do \
	  echo "→ Validating $$f"; \
	  $(SBOM_CLI) validate --input-format json --input-file "$$f"; \
	done

sbom-summary-py:
	@mkdir -p "$(SBOM_DIR)"
	@if ! find "$(SBOM_DIR)" -maxdepth 1 -name '*.cdx.json' -print -quit | grep -q .; then \
	  echo "→ No SBOM files found in $(SBOM_DIR); skipping summary"; \
	  exit 0; \
	fi
	@echo "→ Writing SBOM summary"
	@summary="$(SBOM_DIR)/summary.txt"; : > "$$summary"; \
	if command -v jq >/dev/null 2>&1; then \
	  for f in "$(SBOM_DIR)"/*.cdx.json; do \
	    comps=$$(jq -r '(.components|length) // 0' "$$f"); \
	    echo "$$(basename "$$f")  components=$$comps" >> "$$summary"; \
	  done; \
	else \
	  tmp="$(SBOM_DIR)/_sbom_summary.py"; \
	  echo "import glob, json, os"                                  >  "$$tmp"; \
	  echo "sbom_dir = r'$(SBOM_DIR)'"                              >> "$$tmp"; \
	  echo "for f in glob.glob(os.path.join(sbom_dir, '*.cdx.json')):" >> "$$tmp"; \
	  echo "    try:"                                               >> "$$tmp"; \
	  echo "        with open(f, 'r', encoding='utf-8') as fh:"     >> "$$tmp"; \
	  echo "            d = json.load(fh)"                          >> "$$tmp"; \
	  echo "        comps = len(d.get('components', []) or [])"     >> "$$tmp"; \
	  echo "    except Exception:"                                  >> "$$tmp"; \
	  echo "        comps = '?'"                                    >> "$$tmp"; \
	  echo "    print(os.path.basename(f) + '  components=' + str(comps))" >> "$$tmp"; \
	  python3 "$$tmp" >> "$$summary" || true; \
	  rm -f "$$tmp"; \
	fi; \
	sed -n '1,5p' "$$summary" 2>/dev/null || true

sbom-clean-py:
	@echo "→ Cleaning Python SBOM artifacts"
	@mkdir -p "$(SBOM_DIR)"
	@rm -f \
	  "$(SBOM_DIR)/$(PACKAGE_NAME)-0.0.0-"*.cdx.json \
	  "$(SBOM_DIR)/$(PACKAGE_NAME)--"*.cdx.json || true

# Backward-compatible aliases
sbom: sbom-py
sbom-prod: sbom-prod-py
sbom-dev: sbom-dev-py
sbom-validate: sbom-validate-py
sbom-summary: sbom-summary-py
sbom-clean: sbom-clean-py

##@ Python SBOM
sbom-py:           ## Generate SBOMs for prod/dev (pip-audit → CycloneDX JSON)
sbom-validate-py:  ## Validate generated SBOMs with CycloneDX CLI
sbom-summary-py:   ## Write a brief components summary to artifacts/sbom/summary.txt
sbom-clean-py:     ## Remove stale SBOM artifacts from artifacts/sbom

# -------------------------------
# Python publish
# -------------------------------
DIST_DIR            ?= artifacts/build
PKG_DIST_NAME       ?= bijux_cli
PY                  ?= python3
TWINE               ?= $(PY) -m twine
TWINE_REPOSITORY    ?= pypi
TWINE_USERNAME      ?= __token__
TWINE_PASSWORD      ?= $(PYPI_API_TOKEN)
SKIP_TWINE_CHECK    ?= 0
SKIP_EXISTING       ?= 1

PYTHON_PYPROJECT    ?= crates/bijux-cli-python/pyproject.toml
PKG_VERSION_RAW     := $(shell awk -F'"' '/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ { print $$2; exit }' "$(PYTHON_PYPROJECT)" 2>/dev/null)
PKG_VERSION         := $(if $(strip $(PKG_VERSION_RAW)),$(strip $(PKG_VERSION_RAW)),0.0.0)

.PHONY: \
	publish-py publish-test-py twine-py twine-check-py twine-upload-py twine-upload-test-py verify-test-install-py \
	publish publish-test twine twine-check twine-upload twine-upload-test verify-test-install ensure-dists-py check-version-py

twine-py: publish-py

publish-py: check-version-py build-py twine-check-py twine-upload-py
	@echo "✔ Published Python package $(PKG_DIST_NAME) $(PKG_VERSION) to $(TWINE_REPOSITORY)"

publish-test-py: check-version-py build-py twine-check-py twine-upload-test-py
	@echo "✔ Published Python package $(PKG_DIST_NAME) $(PKG_VERSION) to testpypi"

check-version-py:
	@echo "→ Python package version: $(PKG_VERSION)"
	@[ "$(PKG_VERSION)" != "0.0.0" ] || { echo "✘ PKG_VERSION resolved to 0.0.0"; exit 1; }

ensure-dists-py:
	@echo "→ Verifying artifacts for $(PKG_VERSION) in '$(DIST_DIR)'"
	@test -d "$(DIST_DIR)" || { echo "✘ Dist dir missing: $(DIST_DIR)"; exit 1; }
	@whl=$$(ls "$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-"*.whl 2>/dev/null | head -n1); \
	 sdist="$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION).tar.gz"; \
	 test -n "$$whl" || { echo "✘ Missing wheel: $(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-*.whl"; exit 1; }; \
	 test -f "$$sdist" || { echo "✘ Missing sdist: $$sdist"; exit 1; }; \
	 ls -lh "$$whl" "$$sdist"

twine-check-py: ensure-dists-py
ifeq ($(SKIP_TWINE_CHECK),1)
	@echo "→ Skipping twine check (SKIP_TWINE_CHECK=$(SKIP_TWINE_CHECK))"
else
	@echo "→ Running twine check"
	@whl=$$(ls "$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-"*.whl | head -n1); \
	 sdist="$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION).tar.gz"; \
	 $(TWINE) check "$$whl" "$$sdist"
endif

twine-upload-py: ensure-dists-py
	@echo "→ Uploading $(PKG_DIST_NAME) $(PKG_VERSION) to repository '$(TWINE_REPOSITORY)'"
	@test -n "$(TWINE_PASSWORD)" || { echo "✘ PYPI_API_TOKEN (TWINE_PASSWORD) not set"; exit 1; }
	@whl=$$(ls "$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-"*.whl | head -n1); \
	 sdist="$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION).tar.gz"; \
	 SKIP=""; [ "$(SKIP_EXISTING)" = "1" ] && SKIP="--skip-existing"; \
	 $(TWINE) upload --non-interactive --disable-progress-bar $$SKIP \
	   --repository "$(TWINE_REPOSITORY)" -u "$(TWINE_USERNAME)" -p "$(TWINE_PASSWORD)" \
	   "$$whl" "$$sdist"

twine-upload-test-py:
	@$(MAKE) twine-upload-py TWINE_REPOSITORY=testpypi

verify-test-install-py:
	@echo "→ Verifying installation from TestPyPI"
	@tmp=$$(mktemp -d); \
	$(PY) -m venv $$tmp/venv; \
	$$tmp/venv/bin/pip install -U pip; \
	$$tmp/venv/bin/pip install -i https://test.pypi.org/simple --extra-index-url https://pypi.org/simple bijux-cli==$(PKG_VERSION); \
	$$tmp/venv/bin/bijux --version; \
	echo "✔ TestPyPI install OK"; \
	echo "Temp venv at $$tmp (delete when done)"

# Backward-compatible aliases
twine: twine-py
publish: publish-py
publish-test: publish-test-py
twine-check: twine-check-py
twine-upload: twine-upload-py
twine-upload-test: twine-upload-test-py
verify-test-install: verify-test-install-py

##@ Publish
publish-py:             ## Upload Python release to PyPI (build → validate → upload)
publish-test-py:        ## Upload Python release to TestPyPI (build → validate → upload)
verify-test-install-py: ## Install from TestPyPI into temp venv and run CLI
