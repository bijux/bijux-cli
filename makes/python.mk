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
PYTEST_CACHE_DIR        ?= $(abspath $(TEST_ARTIFACTS_DIR)/.pytest_cache)

PYTEST_INI := $(abspath $(PYTHON_CONFIG_DIR)/pytest.ini)
COVCFG_INI := $(abspath $(PYTHON_CONFIG_DIR)/coveragerc.ini)

PY_RUNTIME_BIN     ?= artifacts/rust/target/debug/bijux
PY_DAG_RUNTIME_BIN ?= artifacts/rust/target/debug/bijux-dag

PYTEST_DEFAULT_MARKER_EXPR ?= not nightly
PYTEST_MARKER_EXPR         ?= $(PYTEST_DEFAULT_MARKER_EXPR)
PYTEST_ADDOPTS ?= -ra --strict-markers --tb=short --cov=bijux_cli_py --cov-branch --cov-config=$(COVCFG_INI) --cov-report=term-missing:skip-covered --cov-report=html:$(abspath $(TEST_ARTIFACTS_DIR)/htmlcov) --cov-report=xml:$(abspath $(TEST_ARTIFACTS_DIR)/coverage.xml) --cov-fail-under=60

# Mirror [tool.security.pip_audit_ignore] in crates/bijux-cli-python/pyproject.toml.
PIP_AUDIT_IGNORE_IDS ?= \
	CVE-2026-3219 \
	PYSEC-2022-42969
PIP_AUDIT_IGNORE_FLAGS := $(foreach id,$(PIP_AUDIT_IGNORE_IDS),--ignore-vuln $(id))
PIP_AUDIT_FLAGS ?= --progress-spinner off --skip-editable $(PIP_AUDIT_IGNORE_FLAGS)

TWINE_REPOSITORY     ?= pypi
PUBLISH_SKIP_EXISTING ?= 1
PUBLISH_BUILD         ?= 1
PYPI_TOKEN_ENV       ?= PYPI_API_TOKEN

.PHONY: python-env fmt-py fmt-check-py lint-py lint-check-py test-py test-unit-py test-nightly-py security-py build-py publish-py

define run_pytest
	@echo "→ Running Python tests on $(PYTHON_TEST_DIR)"
	@mkdir -p "$(TEST_ARTIFACTS_DIR)"
	@echo "→ Building Rust runtime binaries for Python parity tests"
	@cargo build -q --locked \
	  -p bijux-cli --bin bijux \
	  -p bijux-dag-cli --bin bijux-dag
	@echo "   • JUnit XML → $(abspath $(TEST_ARTIFACTS_DIR)/junit.xml)"
	@echo "   • Using pytest → $(PYTEST)"
	@set -euo pipefail; \
	extra_addopts="$(strip $(3))"; \
	status=0; \
	PYTHONPATH="$(abspath $(PYTHON_SRC_DIR))$${PYTHONPATH:+:$${PYTHONPATH}}" \
	BIJUX_BIN="$(abspath $(PY_RUNTIME_BIN))" \
	BIJUX_DAG_BIN="$(abspath $(PY_DAG_RUNTIME_BIN))" \
	$(PYTEST) -c "$(PYTEST_INI)" "$(abspath $(PYTHON_TEST_DIR))" \
	  --junitxml "$(abspath $(TEST_ARTIFACTS_DIR)/junit.xml)" \
	  -o cache_dir="$(PYTEST_CACHE_DIR)" \
	  -o addopts='$(PYTEST_ADDOPTS) -m "$(1)" '$${extra_addopts} || status=$$?; \
	if [ "$$status" -eq 5 ] && [ "$(2)" = "allow-empty" ]; then \
	  echo "→ No tests matched marker expression: $(1)"; \
	  exit 0; \
	fi; \
	exit $$status
	@rm -rf .ruff_cache || true
endef

##@ Python
python-env: install ## Prepare the repo-managed Python virtualenv and tools
	@rm -f "$(PYTHON_SRC_DIR)/bijux_cli_py"/_native*.so || true

fmt-py: python-env ## Run Python formatting with Ruff
	@echo "→ Ruff format"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -o pipefail; \
	$(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)" --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"
	@rm -rf .ruff_cache || true

fmt-check-py: python-env ## Verify Python formatting without modifying files
	@echo "→ Ruff format check"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -o pipefail; \
	$(RUFF) format --check --cache-dir "$(RUFF_CACHE_DIR)" --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format-check.log"
	@rm -rf .ruff_cache || true

lint-py: fmt-py ## Run Python lint fixes with Ruff
	@echo "→ Ruff lint"
	@set -o pipefail; \
	$(RUFF) check --cache-dir "$(RUFF_CACHE_DIR)" --fix --select E,F,I,UP,B,SIM,C4,TID,PERF --ignore E501 --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-check.log"
	@rm -rf .ruff_cache || true

lint-check-py: python-env ## Verify Python lint checks without modifying files
	@echo "→ Ruff lint check"
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -o pipefail; \
	$(RUFF) check --cache-dir "$(RUFF_CACHE_DIR)" --select E,F,I,UP,B,SIM,C4,TID,PERF --ignore E501 --config "$(PYTHON_CONFIG_DIR)/ruff.toml" $(LINT_PATHS) \
	  2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-check-ci.log"
	@rm -rf .ruff_cache || true

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
	@$(PIP_AUDIT) $(PIP_AUDIT_FLAGS) -f json -o "$(SECURITY_ARTIFACTS_DIR)/pip-audit.json"
	@set -o pipefail; \
	$(PIP_AUDIT) $(PIP_AUDIT_FLAGS) \
	  2>&1 | tee "$(SECURITY_ARTIFACTS_DIR)/pip-audit.txt"

build-py: python-env ## Build the Python wheel and source distribution
	@echo "→ Building Python wheel and sdist"
	@mkdir -p "$(BUILD_ARTIFACTS_DIR)"
	@rm -f "$(BUILD_ARTIFACTS_DIR)"/*.whl "$(BUILD_ARTIFACTS_DIR)"/*.tar.gz "$(BUILD_ARTIFACTS_DIR)/twine-check.log" || true
	@set -euo pipefail; \
	build_source="$(PYTHON_PACKAGE_DIR)"; \
	temp_root=""; \
	if [ -n "$(RELEASE_VERSION)" ]; then \
		temp_root="$$(mktemp -d "$${TMPDIR:-/tmp}/bijux-release-tree.XXXXXX")"; \
		trap 'test -n "$${temp_root}" && rm -rf "$${temp_root}"' EXIT; \
		python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$${temp_root}" --version "$(RELEASE_VERSION)" >/dev/null; \
		build_source="$${temp_root}/$(PYTHON_PACKAGE_DIR)"; \
		echo "→ Building from release tree stamped to $(RELEASE_VERSION)"; \
	fi; \
	$(BUILD_PY) --wheel --sdist --outdir "$(BUILD_ARTIFACTS_DIR)" "$${build_source}"
	@set -o pipefail; \
	$(TWINE) check "$(BUILD_ARTIFACTS_DIR)"/* 2>&1 | tee "$(BUILD_ARTIFACTS_DIR)/twine-check.log"

publish-py: python-env ## Publish Python distributions to the configured index
	@token="$${$(PYPI_TOKEN_ENV):-}"; \
	if [ -z "$$token" ]; then \
	  echo "✘ $(PYPI_TOKEN_ENV) is not set"; \
	  exit 1; \
	fi; \
	if [ -n "$(RELEASE_VERSION)" ]; then \
	  echo "→ Publishing release version $(RELEASE_VERSION)"; \
	else \
	 	package_version="$$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p['\''name'\'']: p['\''version'\''] for p in data['\''packages'\'']}; print(pkgs.get('\''bijux-cli-python'\'', '\'''\''))' 2>/dev/null)"; \
	  if [ -z "$$package_version" ]; then \
	    echo "✘ Could not resolve bijux-cli-python version from cargo metadata"; \
	    exit 1; \
	  fi; \
	  case "$$package_version" in \
	    *-*) if [ "$(PUBLISH_ALLOW_PRERELEASE)" != "1" ]; then \
	      echo "✘ Refusing to publish prerelease workspace version $$package_version without RELEASE_VERSION or PUBLISH_ALLOW_PRERELEASE=1"; \
	      exit 1; \
	    fi ;; \
	  esac; \
	  echo "→ Publishing workspace package version $$package_version"; \
	fi; \
	if [ "$(PUBLISH_BUILD)" = "1" ]; then \
	  $(MAKE) --no-print-directory build-py; \
	else \
	  echo "→ Using prebuilt distributions from $(BUILD_ARTIFACTS_DIR)"; \
	fi; \
	SKIP_FLAG=""; \
	if [ "$(PUBLISH_SKIP_EXISTING)" = "1" ]; then SKIP_FLAG="--skip-existing"; fi; \
	mkdir -p "$(BUILD_ARTIFACTS_DIR)"; \
	dist_files=(); \
	for candidate in "$(BUILD_ARTIFACTS_DIR)"/*.whl "$(BUILD_ARTIFACTS_DIR)"/*.tar.gz; do \
	  if [ -e "$$candidate" ]; then \
	    dist_files+=("$$candidate"); \
	  fi; \
	done; \
	if [ "$${#dist_files[@]}" -eq 0 ]; then \
	  echo "✘ No Python distributions found in $(BUILD_ARTIFACTS_DIR)"; \
	  exit 1; \
	fi; \
	$(TWINE) check "$${dist_files[@]}" 2>&1 | tee "$(BUILD_ARTIFACTS_DIR)/twine-check.log"; \
	echo "→ Uploading distributions to $(TWINE_REPOSITORY)"; \
	$(TWINE) upload --non-interactive --disable-progress-bar $$SKIP_FLAG \
	  --repository "$(TWINE_REPOSITORY)" -u "__token__" -p "$$token" \
	  "$${dist_files[@]}"
