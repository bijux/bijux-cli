# Minimal Python lane for bijux-cli-python.
# Supported workflows: lint, test, security, build, publish.

PYTHON_PACKAGE_DIR    ?= crates/bijux-cli-python
PYTHON_SRC_DIR        ?= $(PYTHON_PACKAGE_DIR)/python
PYTHON_TEST_DIR       ?= $(PYTHON_PACKAGE_DIR)/tests/python
PYTHON_CONFIG_DIR     ?= configs/python
PYTHON_PYPROJECT      ?= $(PYTHON_PACKAGE_DIR)/pyproject.toml
CARGO_MANIFEST_PY     ?= $(PYTHON_PACKAGE_DIR)/Cargo.toml

PYTHON_BOOTSTRAP ?= $(shell command -v python3.11 2>/dev/null || command -v python3 2>/dev/null || command -v python 2>/dev/null)
VENV_PYTHON      ?= $(VENV)/bin/python

RUFF       ?= $(ACT)/ruff
PYTEST     ?= $(ACT)/pytest
BANDIT     ?= $(ACT)/bandit
PIP_AUDIT  ?= $(ACT)/pip-audit
BUILD_PY   ?= $(VENV_PYTHON) -m build
TWINE      ?= $(VENV_PYTHON) -m twine

LINT_ARTIFACTS_DIR      ?= artifacts/lint
TEST_ARTIFACTS_DIR      ?= artifacts/test
SECURITY_ARTIFACTS_DIR  ?= artifacts/security
BUILD_ARTIFACTS_DIR     ?= artifacts/build
LINT_PATHS              ?= $(PYTHON_SRC_DIR)/bijux_cli_py
RUFF_CACHE_DIR          ?= $(abspath $(LINT_ARTIFACTS_DIR)/.ruff_cache)
PYTEST_BENCHMARK_DIR    ?= $(abspath $(TEST_ARTIFACTS_DIR)/benchmarks)

PYTEST_INI := $(abspath $(PYTHON_CONFIG_DIR)/pytest.ini)
COVCFG_INI := $(abspath $(PYTHON_CONFIG_DIR)/coveragerc.ini)

PY_RUNTIME_BIN ?= artifacts/rust/target/debug/bijux

PYTEST_ADDOPTS ?= -ra --strict-markers --tb=short -m "not nightly and not slow" --cov=bijux_cli_py --cov-branch --cov-config=$(COVCFG_INI) --cov-report=term-missing:skip-covered --cov-report=html:$(abspath $(TEST_ARTIFACTS_DIR)/htmlcov) --cov-report=xml:$(abspath $(TEST_ARTIFACTS_DIR)/coverage.xml) --cov-fail-under=60

# Mirrors [tool.security.pip_audit_ignore] in crates/bijux-cli-python/pyproject.toml.
PIP_AUDIT_IGNORE_IDS ?= \
	PYSEC-2022-42969
PIP_AUDIT_IGNORE_FLAGS := $(foreach id,$(PIP_AUDIT_IGNORE_IDS),--ignore-vuln $(id))

TWINE_REPOSITORY     ?= pypi
PUBLISH_SKIP_EXISTING ?= 1
PYPI_TOKEN_ENV       ?= PYPI_API_TOKEN

PYTHON_DEV_TOOLS ?= \
	pytest pytest-cov pytest-asyncio pytest-timeout pytest-rerunfailures pytest-benchmark pytest-mock \
	hypothesis hypothesis-jsonschema pexpect \
	ruff bandit pip-audit \
	build twine maturin

PY_VERSION_RAW := $(shell awk -F'"' '/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ { print $$2; exit }' "$(PYTHON_PYPROJECT)" 2>/dev/null)
RUST_VERSION_RAW := $(shell awk -F'"' '/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/ { print $$2; exit }' "$(CARGO_MANIFEST_PY)" 2>/dev/null)
PY_VERSION := $(if $(strip $(PY_VERSION_RAW)),$(strip $(PY_VERSION_RAW)),0.0.0)
RUST_VERSION := $(if $(strip $(RUST_VERSION_RAW)),$(strip $(RUST_VERSION_RAW)),0.0.0)

.PHONY: python-env-py fmt-py lint-py test-py security-py build-py publish-py lint test security build publish

python-env-py:
	@set -euo pipefail; \
	bootstrap="$(PYTHON_BOOTSTRAP)"; \
	if [ -z "$$bootstrap" ]; then \
	  echo "✘ Python 3.11+ is required but no Python interpreter was found"; \
	  exit 1; \
	fi; \
	if [ -d ".venv" ] && [ "$(VENV)" != ".venv" ]; then \
	  if [ -d "$(VENV)" ]; then \
	    echo "→ Removing legacy root .venv (using $(VENV))"; \
	    rm -rf ".venv"; \
	  else \
	    echo "→ Migrating legacy .venv to $(VENV)"; \
	    mkdir -p "$(dir $(VENV))"; \
	    mv ".venv" "$(VENV)"; \
	  fi; \
	fi; \
	if [ ! -x "$(VENV)/bin/python" ]; then \
	  echo "→ Creating virtualenv with '$$bootstrap' ..."; \
	  mkdir -p "$(dir $(VENV))"; \
	  "$$bootstrap" -m venv "$(VENV)"; \
	fi; \
	if ! "$(VENV)/bin/python" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else 1)'; then \
	  old_ver="$$("$(VENV)/bin/python" -c 'import sys; print(f"{sys.version_info[0]}.{sys.version_info[1]}")' 2>/dev/null || echo unknown)"; \
	  echo "→ Recreating $(VENV) with Python >=3.11 (found $$old_ver)"; \
	  rm -rf "$(VENV)"; \
	  "$$bootstrap" -m venv "$(VENV)"; \
	fi; \
	"$(VENV)/bin/python" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else 1)' || { \
	  echo "✘ Active virtualenv is not Python 3.11+"; \
	  exit 1; \
	}; \
	need_install=0; \
	for tool in "$(RUFF)" "$(PYTEST)" "$(BANDIT)" "$(PIP_AUDIT)"; do \
	  [ -x "$$tool" ] || need_install=1; \
	done; \
	if [ $$need_install -eq 1 ]; then \
	  echo "→ Installing Python dev dependencies into $(VENV)"; \
	  "$(VENV)/bin/python" -m pip install --upgrade pip setuptools wheel; \
	  "$(VENV)/bin/python" -m pip install $(PYTHON_DEV_TOOLS); \
	fi
	@rm -f "$(PYTHON_SRC_DIR)/bijux_cli_py"/_native*.so || true

fmt-py: python-env-py
	@echo "→ Ruff format"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@$(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)" --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"
	@rm -rf .ruff_cache .benchmark .benchmarks || true

lint-py: fmt-py
	@echo "→ Ruff lint"
	@$(RUFF) check --cache-dir "$(RUFF_CACHE_DIR)" --fix --select E,F,I,UP,B,SIM,C4,TID,PERF --ignore E501 --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-check.log"
	@rm -rf .ruff_cache .benchmark .benchmarks || true

test-py: python-env-py
	@echo "→ Running Python test suite on $(PYTHON_TEST_DIR)"
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
	PYTHONPATH="$(abspath $(PYTHON_SRC_DIR))$${PYTHONPATH:+:$${PYTHONPATH}}" \
	HYPOTHESIS_DATABASE_DIRECTORY="$(abspath $(TEST_ARTIFACTS_DIR)/hypothesis)" \
	BIJUX_BIN="$(abspath $(PY_RUNTIME_BIN))" \
	$(PYTEST) -c "$(PYTEST_INI)" "$(abspath $(PYTHON_TEST_DIR))" \
	  --junitxml "$(abspath $(TEST_ARTIFACTS_DIR)/junit.xml)" \
	  -o cache_dir="$(abspath $(TEST_ARTIFACTS_DIR)/.pytest_cache)" \
	  -o addopts='$(PYTEST_ADDOPTS)' \
	  $$BENCH_FLAGS
	@rm -rf .benchmarks .benchmark .ruff_cache || true

security-py: python-env-py
	@echo "→ Bandit (medium/high severity)"
	@mkdir -p "$(SECURITY_ARTIFACTS_DIR)"
	@$(BANDIT) -r "$(PYTHON_SRC_DIR)/bijux_cli_py" -ll -f json -o "$(SECURITY_ARTIFACTS_DIR)/bandit.json"
	@$(BANDIT) -r "$(PYTHON_SRC_DIR)/bijux_cli_py" -ll \
	  2>&1 | tee "$(SECURITY_ARTIFACTS_DIR)/bandit.txt"
	@echo "→ pip-audit"
	@$(PIP_AUDIT) --progress-spinner off $(PIP_AUDIT_IGNORE_FLAGS) -f json -o "$(SECURITY_ARTIFACTS_DIR)/pip-audit.json"
	@$(PIP_AUDIT) --progress-spinner off $(PIP_AUDIT_IGNORE_FLAGS) \
	  2>&1 | tee "$(SECURITY_ARTIFACTS_DIR)/pip-audit.txt"

build-py: python-env-py
	@echo "→ Building Python wheel and sdist"
	@mkdir -p "$(BUILD_ARTIFACTS_DIR)"
	@rm -f "$(BUILD_ARTIFACTS_DIR)"/*.whl "$(BUILD_ARTIFACTS_DIR)"/*.tar.gz "$(BUILD_ARTIFACTS_DIR)/twine-check.log" || true
	@$(BUILD_PY) --wheel --sdist --outdir "$(BUILD_ARTIFACTS_DIR)" "$(PYTHON_PACKAGE_DIR)"
	@$(TWINE) check "$(BUILD_ARTIFACTS_DIR)"/* 2>&1 | tee "$(BUILD_ARTIFACTS_DIR)/twine-check.log"

publish-py: python-env-py
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

# Compatibility aliases.
lint: lint-py
test: test-py
security: security-py
build: build-py
publish: publish-py
