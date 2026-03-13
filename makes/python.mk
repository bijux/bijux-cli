# Python commands for bijux-cli-python.
# Supported workflows: lint, test, security, build, and publish.

PYTHON_PACKAGE_DIR    ?= crates/bijux-cli-python
PYTHON_SRC_DIR        ?= $(PYTHON_PACKAGE_DIR)/python
PYTHON_TEST_DIR       ?= $(PYTHON_PACKAGE_DIR)/tests/python
PYTHON_CONFIG_DIR     ?= configs/python
PYTHON_PYPROJECT      ?= $(PYTHON_PACKAGE_DIR)/pyproject.toml
CARGO_MANIFEST_PY     ?= $(PYTHON_PACKAGE_DIR)/Cargo.toml

VENV_PYTHON      ?= $(VENV)/bin/python

RUFF       ?= $(ACT)/ruff
PYTEST     ?= $(ACT)/pytest
BANDIT     ?= $(ACT)/bandit
PIP_AUDIT  ?= $(ACT)/pip-audit
BUILD_PY   ?= $(VENV_PYTHON) -m build
TWINE      ?= $(VENV_PYTHON) -m twine

PYTHON_ARTIFACTS_DIR    ?= artifacts/python
LINT_ARTIFACTS_DIR      ?= $(PYTHON_ARTIFACTS_DIR)/lint
TEST_ARTIFACTS_DIR      ?= $(PYTHON_ARTIFACTS_DIR)/test
SECURITY_ARTIFACTS_DIR  ?= $(PYTHON_ARTIFACTS_DIR)/security
BUILD_ARTIFACTS_DIR     ?= $(PYTHON_ARTIFACTS_DIR)/build
LINT_PATHS              ?= $(PYTHON_SRC_DIR)/bijux_cli_py
RUFF_CACHE_DIR          ?= $(abspath $(LINT_ARTIFACTS_DIR)/.ruff_cache)
PYTEST_BENCHMARK_DIR    ?= $(abspath $(TEST_ARTIFACTS_DIR)/benchmarks)
PYTEST_CACHE_DIR        ?= $(abspath $(TEST_ARTIFACTS_DIR)/.pytest_cache)

PYTEST_INI := $(abspath $(PYTHON_CONFIG_DIR)/pytest.ini)
COVCFG_INI := $(abspath $(PYTHON_CONFIG_DIR)/coveragerc.ini)

PY_RUNTIME_BIN ?= artifacts/rust/target/debug/bijux

PYTEST_DEFAULT_MARKER_EXPR ?= not nightly and not slow
PYTEST_MARKER_EXPR         ?= $(PYTEST_DEFAULT_MARKER_EXPR)
PYTEST_ADDOPTS ?= -ra --strict-markers --tb=short --cov=bijux_cli_py --cov-branch --cov-config=$(COVCFG_INI) --cov-report=term-missing:skip-covered --cov-report=html:$(abspath $(TEST_ARTIFACTS_DIR)/htmlcov) --cov-report=xml:$(abspath $(TEST_ARTIFACTS_DIR)/coverage.xml) --cov-fail-under=60

# Mirror [tool.security.pip_audit_ignore] in crates/bijux-cli-python/pyproject.toml.
PIP_AUDIT_IGNORE_IDS ?= \
	PYSEC-2022-42969
PIP_AUDIT_IGNORE_FLAGS := $(foreach id,$(PIP_AUDIT_IGNORE_IDS),--ignore-vuln $(id))

TWINE_REPOSITORY     ?= pypi
PUBLISH_SKIP_EXISTING ?= 1
PYPI_TOKEN_ENV       ?= PYPI_API_TOKEN

PY_VERSION_RAW := $(shell awk -F'"' '/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ { print $$2; exit }' "$(PYTHON_PYPROJECT)" 2>/dev/null)
RUST_VERSION_RAW := $(shell awk -F'"' '/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ { print $$2; exit }' "$(CARGO_MANIFEST_PY)" 2>/dev/null)
PY_VERSION := $(if $(strip $(PY_VERSION_RAW)),$(strip $(PY_VERSION_RAW)),0.0.0)
RUST_VERSION := $(if $(strip $(RUST_VERSION_RAW)),$(strip $(RUST_VERSION_RAW)),0.0.0)

.PHONY: python-env python-env-py fmt-py fmt-check-py lint-py lint-check-py test-py test-unit-py test-nightly-py security-py build-py publish-py

define run_pytest
	@echo "→ Running Python tests on $(PYTHON_TEST_DIR)"
	@mkdir -p "$(TEST_ARTIFACTS_DIR)" "$(TEST_ARTIFACTS_DIR)/hypothesis" "$(PYTEST_BENCHMARK_DIR)"
	@if [ ! -x "$(PY_RUNTIME_BIN)" ]; then \
	  echo "→ Building Rust runtime binary for Python parity tests"; \
	  cargo build -q -p bijux-cli --bin bijux; \
	fi
	@echo "   • JUnit XML → $(abspath $(TEST_ARTIFACTS_DIR)/junit.xml)"
	@echo "   • Hypothesis DB → $(abspath $(TEST_ARTIFACTS_DIR)/hypothesis)"
	@echo "   • Using pytest → $(PYTEST)"
	@set -euo pipefail; \
	BENCH_FLAGS=""; \
	if "$(PYTEST)" -q --help 2>/dev/null | grep -q -- '--benchmark-storage'; then \
	  BENCH_FLAGS="--benchmark-storage=file://$(PYTEST_BENCHMARK_DIR)"; \
	fi; \
	extra_addopts="$(strip $(3))"; \
	status=0; \
	PYTHONPATH="$(abspath $(PYTHON_SRC_DIR))$${PYTHONPATH:+:$${PYTHONPATH}}" \
	HYPOTHESIS_DATABASE_DIRECTORY="$(abspath $(TEST_ARTIFACTS_DIR)/hypothesis)" \
	BIJUX_BIN="$(abspath $(PY_RUNTIME_BIN))" \
	$(PYTEST) -c "$(PYTEST_INI)" "$(abspath $(PYTHON_TEST_DIR))" \
	  --junitxml "$(abspath $(TEST_ARTIFACTS_DIR)/junit.xml)" \
	  -o cache_dir="$(PYTEST_CACHE_DIR)" \
	  -o addopts='$(PYTEST_ADDOPTS) -m "$(1)" '$${extra_addopts} \
	  $$BENCH_FLAGS || status=$$?; \
	if [ "$$status" -eq 5 ] && [ "$(2)" = "allow-empty" ]; then \
	  echo "→ No tests matched marker expression: $(1)"; \
	  exit 0; \
	fi; \
	exit $$status
	@rm -rf .benchmarks .benchmark .ruff_cache || true
endef

##@ Python
python-env: install ## Prepare the repo-managed Python virtualenv and tools
	@rm -f "$(PYTHON_SRC_DIR)/bijux_cli_py"/_native*.so || true

python-env-py: python-env ## Run the legacy alias for python-env

fmt-py: python-env ## Run Python formatting with Ruff
	@echo "→ Ruff format"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -o pipefail; \
	$(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)" --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"
	@rm -rf .ruff_cache .benchmark .benchmarks || true

fmt-check-py: python-env ## Verify Python formatting without modifying files
	@echo "→ Ruff format check"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -o pipefail; \
	$(RUFF) format --check --cache-dir "$(RUFF_CACHE_DIR)" --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format-check.log"
	@rm -rf .ruff_cache .benchmark .benchmarks || true

lint-py: fmt-py ## Run Python lint fixes with Ruff
	@echo "→ Ruff lint"
	@set -o pipefail; \
	$(RUFF) check --cache-dir "$(RUFF_CACHE_DIR)" --fix --select E,F,I,UP,B,SIM,C4,TID,PERF --ignore E501 --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-check.log"
	@rm -rf .ruff_cache .benchmark .benchmarks || true

lint-check-py: python-env ## Verify Python lint checks without modifying files
	@echo "→ Ruff lint check"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -o pipefail; \
	$(RUFF) check --cache-dir "$(RUFF_CACHE_DIR)" --select E,F,I,UP,B,SIM,C4,TID,PERF --ignore E501 --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-check-ci.log"
	@rm -rf .ruff_cache .benchmark .benchmarks || true

test-py: python-env ## Run the default Python test suite
	$(call run_pytest,$(PYTEST_MARKER_EXPR),strict,)

test-unit-py: python-env ## Run Python tests marked unit
	$(call run_pytest,unit,allow-empty,--no-cov)

test-nightly-py: python-env ## Run Python tests marked nightly
	$(call run_pytest,nightly,allow-empty,--no-cov)

security-py: python-env ## Run Python security checks
	@echo "→ Bandit (medium/high severity)"
	@mkdir -p "$(SECURITY_ARTIFACTS_DIR)"
	@$(BANDIT) -r "$(PYTHON_SRC_DIR)/bijux_cli_py" -ll -f json -o "$(SECURITY_ARTIFACTS_DIR)/bandit.json"
	@set -o pipefail; \
	$(BANDIT) -r "$(PYTHON_SRC_DIR)/bijux_cli_py" -ll \
	  2>&1 | tee "$(SECURITY_ARTIFACTS_DIR)/bandit.txt"
	@echo "→ pip-audit"
	@$(PIP_AUDIT) --progress-spinner off $(PIP_AUDIT_IGNORE_FLAGS) -f json -o "$(SECURITY_ARTIFACTS_DIR)/pip-audit.json"
	@set -o pipefail; \
	$(PIP_AUDIT) --progress-spinner off $(PIP_AUDIT_IGNORE_FLAGS) \
	  2>&1 | tee "$(SECURITY_ARTIFACTS_DIR)/pip-audit.txt"

build-py: python-env ## Build the Python wheel and source distribution
	@echo "→ Building Python wheel and sdist"
	@mkdir -p "$(BUILD_ARTIFACTS_DIR)"
	@rm -f "$(BUILD_ARTIFACTS_DIR)"/*.whl "$(BUILD_ARTIFACTS_DIR)"/*.tar.gz "$(BUILD_ARTIFACTS_DIR)/twine-check.log" || true
	@$(BUILD_PY) --wheel --sdist --outdir "$(BUILD_ARTIFACTS_DIR)" "$(PYTHON_PACKAGE_DIR)"
	@set -o pipefail; \
	$(TWINE) check "$(BUILD_ARTIFACTS_DIR)"/* 2>&1 | tee "$(BUILD_ARTIFACTS_DIR)/twine-check.log"

publish-py: python-env ## Publish Python distributions to the configured index
	@echo "→ Validating Python/Rust package version parity"
	@[ "$(PY_VERSION)" != "0.0.0" ] || { echo "✘ Python package version resolved to 0.0.0"; exit 1; }
	@[ "$(RUST_VERSION)" != "0.0.0" ] || { echo "✘ Rust crate version resolved to 0.0.0"; exit 1; }
	@[ "$(PY_VERSION)" = "$(RUST_VERSION)" ] || { \
	  echo "✘ Version drift: pyproject.toml ($(PY_VERSION)) != Cargo.toml ($(RUST_VERSION))"; \
	  exit 1; \
	}
	@token="$${$(PYPI_TOKEN_ENV):-}"; \
	if [ -z "$$token" ]; then \
	  echo "✘ $(PYPI_TOKEN_ENV) is not set"; \
	  exit 1; \
	fi; \
	$(MAKE) --no-print-directory build-py; \
	SKIP_FLAG=""; \
	if [ "$(PUBLISH_SKIP_EXISTING)" = "1" ]; then SKIP_FLAG="--skip-existing"; fi; \
	echo "→ Uploading distributions to $(TWINE_REPOSITORY)"; \
	$(TWINE) upload --non-interactive --disable-progress-bar $$SKIP_FLAG \
	  --repository "$(TWINE_REPOSITORY)" -u "__token__" -p "$$token" \
	  "$(BUILD_ARTIFACTS_DIR)"/*
