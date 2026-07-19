# Root make defaults, shared environment, and user-facing targets.

# Core configuration.
.DELETE_ON_ERROR:
.DEFAULT_GOAL         := all
.SHELLFLAGS           := -eu -o pipefail -c
SHELL                 := bash
PYTHON                ?= $(shell command -v python3.11 2>/dev/null || command -v python3 2>/dev/null || command -v python 2>/dev/null)
RELEASE_VERSION       ?=
RELEASE_TREE_SCRIPT   ?= .github/scripts/prepare_release_tree.py
PUBLISH_ALLOW_PRERELEASE ?= 0
VENV                  := artifacts/python/.venv
VENV_PYTHON           := $(VENV)/bin/python
ACT                   := $(VENV)/bin
RM                    := rm -rf
PROFRAW_DIR           := artifacts/rust/coverage/profraw
LLVM_PROFILE_FILE     ?= $(abspath $(PROFRAW_DIR)/default_%m_%p.profraw)
BIJUX_RUNTIME_BIN     ?= bijux
PYTHON_EDITABLE_EXTRAS ?= test,lint,security,docs,build
PYTHON_EDITABLE_SPEC  ?= ./crates/bijux-cli-python[$(PYTHON_EDITABLE_EXTRAS)]
PYTHON_INSTALL_ARTIFACTS_DIR ?= artifacts/python/install
PYTHON_BYTECODE_DIR ?= artifacts/python/pycache
PIP_BOOTSTRAP_LOG     ?= $(abspath $(PYTHON_INSTALL_ARTIFACTS_DIR)/pip-bootstrap.log)
PIP_EDITABLE_LOG      ?= $(abspath $(PYTHON_INSTALL_ARTIFACTS_DIR)/pip-editable.log)
export PYTHONPYCACHEPREFIX := $(abspath $(PYTHON_BYTECODE_DIR))

.NOTPARALLEL: all clean

##@ Core
$(VENV):
	@mkdir -p "$(dir $(VENV))"
	@if [ -d ".venv" ] && [ "$(VENV)" != ".venv" ]; then \
	  if [ -d "$(VENV)" ]; then \
	    echo "→ Removing legacy root .venv (using $(VENV))"; \
	    rm -rf ".venv"; \
	  else \
	    echo "→ Migrating legacy .venv to $(VENV)"; \
	    mv ".venv" "$(VENV)"; \
	  fi; \
	fi
	@if [ ! -x "$(VENV_PYTHON)" ]; then \
	  echo "→ Creating virtualenv with '$$(which $(PYTHON))' ..."; \
	  $(PYTHON) -m venv $(VENV); \
	fi

install: $(VENV) ## Install the project into the repo-managed virtualenv under artifacts/
	@if [ -x "$(VENV_PYTHON)" ] && ! "$(VENV_PYTHON)" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else 1)'; then \
	  echo "→ Recreating $(VENV) with Python >=3.11"; \
	  $(RM) "$(VENV)"; \
	  $(PYTHON) -m venv "$(VENV)"; \
	fi
	@mkdir -p "$(PYTHON_INSTALL_ARTIFACTS_DIR)"
	@echo "→ Syncing Python packaging tools"
	@set -euo pipefail; \
	recreate_venv() { \
	  stale_venv="$(VENV).stale.$$"; \
	  if [ -d "$(VENV)" ]; then \
	    mv "$(VENV)" "$${stale_venv}" 2>/dev/null || true; \
	  fi; \
	  $(PYTHON) -m venv "$(VENV)"; \
	  if [ -n "$${stale_venv:-}" ] && [ -d "$${stale_venv}" ]; then \
	    $(RM) "$${stale_venv}" || true; \
	  fi; \
	}; \
	bootstrap_python_tools() { \
	  $(VENV_PYTHON) -m pip install --disable-pip-version-check --quiet --upgrade pip setuptools wheel >"$(PIP_BOOTSTRAP_LOG)" 2>&1; \
	}; \
	if bootstrap_python_tools; then \
	  echo "✓ Python packaging tools ready"; \
	else \
	  echo "→ Recreating $(VENV) after packaging tool sync failure"; \
	  recreate_venv; \
	  if bootstrap_python_tools; then \
	    echo "✓ Python packaging tools ready"; \
	  else \
	    echo "✘ Failed to sync Python packaging tools"; \
	    cat "$(PIP_BOOTSTRAP_LOG)"; \
	    exit 1; \
	  fi; \
	fi
	@echo "→ Syncing editable Python package"
	@set -euo pipefail; \
	recreate_venv() { \
	  stale_venv="$(VENV).stale.$$"; \
	  if [ -d "$(VENV)" ]; then \
	    mv "$(VENV)" "$${stale_venv}" 2>/dev/null || true; \
	  fi; \
	  $(PYTHON) -m venv "$(VENV)"; \
	  if [ -n "$${stale_venv:-}" ] && [ -d "$${stale_venv}" ]; then \
	    $(RM) "$${stale_venv}" || true; \
	  fi; \
	}; \
	install_editable_package() { \
	  $(VENV_PYTHON) -m pip install --disable-pip-version-check --quiet -e "$(PYTHON_EDITABLE_SPEC)" >"$(PIP_EDITABLE_LOG)" 2>&1; \
	}; \
	if install_editable_package; then \
	  echo "✓ Editable Python package ready"; \
	else \
	  echo "→ Recreating $(VENV) after editable package sync failure"; \
	  recreate_venv; \
	  if $(VENV_PYTHON) -m pip install --disable-pip-version-check --quiet --upgrade pip setuptools wheel >"$(PIP_BOOTSTRAP_LOG)" 2>&1 && install_editable_package; then \
	    echo "✓ Editable Python package ready"; \
	  else \
	    echo "✘ Failed to sync editable Python package"; \
	    cat "$(PIP_EDITABLE_LOG)"; \
	    exit 1; \
	  fi; \
	fi

bootstrap: install ## Prepare the local development environment
.PHONY: bootstrap

clean: ## Remove the virtualenv, caches, build outputs, and artifacts
	@$(MAKE) clean-soft
	@echo "→ Cleaning ($(VENV)) ..."
	@if [ -d "$(VENV)" ]; then \
	  $(RM) "$(VENV)" || true; \
	  if [ -d "$(VENV)" ]; then \
	    echo "→ Retrying venv cleanup ($(VENV)) ..."; \
	    sleep 1; \
	    $(RM) "$(VENV)" || true; \
	  fi; \
	  if [ -d "$(VENV)" ]; then \
	    echo "→ Warning: could not fully remove $(VENV); continuing."; \
	  fi; \
	fi
	@$(RM) .venv .venv*/

clean-soft: ## Remove generated outputs and keep the repo-managed virtualenv under artifacts/
	@echo "→ Cleaning (keeping $(VENV)) ..."
	@$(RM) \
	  .pytest_cache htmlcov coverage.xml dist build *.egg-info demo .tmp_home \
	  .ruff_cache .coverage.* .coverage \
	  spec.json openapitools.json node_modules session.sqlite site \
	  usage_test usage_test_artifacts .cache default_*.profraw || true
	@if [ -d artifacts/rust ]; then \
	  find artifacts/rust -mindepth 1 -maxdepth 1 -exec rm -rf {} +; \
	fi
	@if [ -d artifacts/python ]; then \
	  find artifacts/python -mindepth 1 -maxdepth 1 ! -name '.venv' -exec rm -rf {} +; \
	fi
	@if [ -d artifacts/docs ]; then \
	  rm -rf artifacts/docs/site artifacts/docs/.cache artifacts/docs/docs/artifacts; \
	fi
	@find . -path "./$(VENV)" -prune -o -type d -name '__pycache__' -exec $(RM) {} +

all: fmt lint security test build ## Run quality checks and build distributions
	@echo "✔ All targets completed"

fmt: fmt-rs ## Run Rust formatting checks
lint: lint-rs ## Run Rust lint checks
test: test-release-rs test-py ## Run the required Rust release lane and Python test suites
test-slow: test-slow-rs ## Run governed slow Rust tests
test-all: test-all-rs ## Run full Rust tests including ignored tests
audit: audit-rs ## Run Rust dependency and advisory audits
security: audit-rs security-py ## Run Rust and Python security checks
build: build-py ## Build Python distribution packages
.PHONY: fmt

fmt lint test security docs build: | bootstrap

env: ## Show the effective make environment
	@printf '%s\n' \
	  "PYTHON=$(PYTHON)" \
	  "VENV=$(VENV)" \
	  "VENV_PYTHON=$(VENV_PYTHON)" \
	  "ACT=$(ACT)" \
	  "BIJUX_RUNTIME_BIN=$(BIJUX_RUNTIME_BIN)" \
	  "PYTHON_EDITABLE_SPEC=$(PYTHON_EDITABLE_SPEC)"

help: ## Show the available targets
	@awk 'BEGIN{FS=":.*##"; OFS="";} \
	  /^##@/ {gsub(/^##@ */,""); print "\n\033[1m" $$0 "\033[0m"; next} \
	  /^[a-zA-Z0-9_.-]+:.*##/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' \
	  $(MAKEFILE_LIST)
.PHONY: help
