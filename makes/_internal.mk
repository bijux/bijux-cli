# Root make defaults, shared environment, and user-facing targets.

# Core configuration.
.DELETE_ON_ERROR:
.DEFAULT_GOAL         := all
.SHELLFLAGS           := -eu -o pipefail -c
SHELL                 := bash
PYTHON                ?= $(shell command -v python3.11 2>/dev/null || command -v python3 2>/dev/null || command -v python 2>/dev/null)
VENV                  := artifacts/python/.venv
VENV_PYTHON           := $(VENV)/bin/python
ACT                   := $(VENV)/bin
RM                    := rm -rf
PROFRAW_DIR           := artifacts/rust/coverage/profraw
LLVM_PROFILE_FILE     ?= $(abspath $(PROFRAW_DIR)/default_%m_%p.profraw)
BIJUX_RUNTIME_BIN     ?= bijux
PYTHON_EDITABLE_SPEC  ?= ./crates/bijux-cli-python[dev]

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

install: $(VENV) ## Install the project into the artifact-scoped virtualenv
	@if [ -x "$(VENV_PYTHON)" ] && ! "$(VENV_PYTHON)" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else 1)'; then \
	  echo "→ Recreating $(VENV) with Python >=3.11"; \
	  $(RM) "$(VENV)"; \
	  $(PYTHON) -m venv "$(VENV)"; \
	fi
	@echo "→ Installing dependencies..."
	@$(VENV_PYTHON) -m pip install --upgrade pip setuptools wheel
	@$(VENV_PYTHON) -m pip install -e "$(PYTHON_EDITABLE_SPEC)"

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

clean-soft: ## Remove generated outputs and keep the artifact-scoped virtualenv
	@echo "→ Cleaning (keeping $(VENV)) ..."
	@$(RM) \
	  .pytest_cache htmlcov coverage.xml dist build *.egg-info demo .tmp_home \
	  .ruff_cache .mypy_cache .hypothesis .coverage.* .coverage .benchmarks \
	  spec.json openapitools.json node_modules .mutmut-cache session.sqlite site \
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

fmt: fmt-rs fmt-py ## Run Rust and Python formatters
lint: lint-rs lint-py ## Run Rust and Python lint checks
test: test-rs test-py ## Run Rust and Python test suites
security: audit-rs security-py ## Run Rust and Python security checks
build: build-py ## Build Python distribution packages
.PHONY: fmt

fmt lint test security docs build: | bootstrap

dev-cli-status: ## Show the maintainer status report
	@mkdir -p "$(PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" cargo run -q -p bijux-cli --bin "$(BIJUX_RUNTIME_BIN)" -- dev cli status --text

dev-cli-crate-health: ## Show the maintainer crate health report
	@mkdir -p "$(PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" cargo run -q -p bijux-cli --bin "$(BIJUX_RUNTIME_BIN)" -- dev cli crate-health --text

dev-cli-parity: ## Show the maintainer parity report
	@mkdir -p "$(PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" cargo run -q -p bijux-cli --bin "$(BIJUX_RUNTIME_BIN)" -- dev cli parity --text

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
