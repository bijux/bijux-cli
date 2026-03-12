ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

# Core config
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

.NOTPARALLEL: all clean

# Modular includes
include $(ROOT_MK_DIR)/macro.mk
include $(ROOT_MK_DIR)/dev-rust.mk
include $(ROOT_MK_DIR)/dev-python.mk
include $(ROOT_MK_DIR)/docs.mk

BIJUX_RUNTIME_BIN ?= bijux

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

install: $(VENV) ## Install project in editable mode into artifact-scoped virtualenv
	@if [ -x "$(VENV_PYTHON)" ] && ! "$(VENV_PYTHON)" -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else 1)'; then \
	  echo "→ Recreating $(VENV) with Python >=3.11"; \
	  $(RM) "$(VENV)"; \
	  $(PYTHON) -m venv "$(VENV)"; \
	fi
	@echo "→ Installing dependencies..."
	@$(VENV_PYTHON) -m pip install --upgrade pip setuptools wheel
	@$(VENV_PYTHON) -m pip install -e "./crates/bijux-cli-python[dev]"

bootstrap: $(VENV) ## Setup environment
.PHONY: bootstrap

clean: ## Remove virtualenv, caches, build, and artifacts
	@$(MAKE) clean-soft
	@echo "→ Cleaning ($(VENV)) ..."
	@$(RM) $(VENV)
	@$(RM) .venv .venv*/

clean-soft: ## Remove build artifacts but keep artifact-scoped virtualenv
	@echo "→ Cleaning (keeping $(VENV)) ..."
	@$(RM) \
	  .pytest_cache htmlcov coverage.xml dist build *.egg-info .tox demo .tmp_home \
	  .ruff_cache .mypy_cache .hypothesis .coverage.* .coverage .benchmarks \
	  spec.json openapitools.json node_modules .mutmut-cache session.sqlite site \
	  docs/reference usage_test usage_test_artifacts .cache default_*.profraw || true
	@if [ -d artifacts ]; then \
	  find artifacts -mindepth 1 -maxdepth 1 ! -name python -exec rm -rf {} +; \
	  if [ -d artifacts/python ]; then \
	    find artifacts/python -mindepth 1 -maxdepth 1 ! -name '.venv' -exec rm -rf {} +; \
	  fi; \
	fi
	@find . -type d -name '__pycache__' -exec $(RM) {} +

all: clean install test lint security docs build ## Run full pipeline (clean → build)
	@echo "✔ All targets completed"

# Run independent checks in parallel
lint test security docs build: | bootstrap
.NOTPARALLEL:

dev-cli-status: ## Show maintainer status report via bijux dev cli
	@mkdir -p "$(PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" cargo run -q -p bijux-cli --bin "$(BIJUX_RUNTIME_BIN)" -- dev cli status --text

dev-cli-crate-health: ## Show crate health and duplication report via bijux dev cli
	@mkdir -p "$(PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" cargo run -q -p bijux-cli --bin "$(BIJUX_RUNTIME_BIN)" -- dev cli crate-health --text

dev-cli-parity: ## Show parity summary via bijux dev cli
	@mkdir -p "$(PROFRAW_DIR)"
	@LLVM_PROFILE_FILE="$(LLVM_PROFILE_FILE)" cargo run -q -p bijux-cli --bin "$(BIJUX_RUNTIME_BIN)" -- dev cli parity --text

# Utilities
define run_tool
	printf "→ %s %s\n" "$(1)" "$$file"; \
	OUT=`$(2) "$$file" 2>&1`; \
	if [ $$? -eq 0 ]; then \
		printf "  ✔ %s OK\n" "$(1)"; \
	else \
		printf "  ✘ %s failed:\n" "$(1)"; \
		printf "%s\n" "$$OUT" | head -10; \
	fi
endef

define read_pyproject_version
$(strip $(shell \
  python3 -c 'import tomllib; \
  print(tomllib.load(open("crates/bijux-cli-python/pyproject.toml","rb"))["project"]["version"])' \
  2>/dev/null || echo 0.0.0 \
))
endef

help: ## Show this help
	@awk 'BEGIN{FS=":.*##"; OFS="";} \
	  /^##@/ {gsub(/^##@ */,""); print "\n\033[1m" $$0 "\033[0m"; next} \
	  /^[a-zA-Z0-9_.-]+:.*##/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' \
	  $(MAKEFILE_LIST)
.PHONY: help
