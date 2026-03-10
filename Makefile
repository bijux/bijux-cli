# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

# Core Config
.DELETE_ON_ERROR:
.DEFAULT_GOAL         := all
.SHELLFLAGS           := -eu -o pipefail -c
SHELL                 := bash
PYTHON                := python3
VENV                  := .venv
VENV_PYTHON           := $(VENV)/bin/python
ACT                   := $(VENV)/bin
RM                    := rm -rf

.NOTPARALLEL: all clean

# Modular Includes
include makefiles/api.mk
include makefiles/build.mk
include makefiles/changelog.mk
include makefiles/citation.mk
include makefiles/cargo-rs.mk
include makefiles/dictionary.mk
include makefiles/docs.mk
include makefiles/lint.mk
include makefiles/mutation.mk
include makefiles/quality.mk
include makefiles/sbom.mk
include makefiles/security.mk
include makefiles/test.mk
include makefiles/publish.mk
include makefiles/hooks.mk

# Environment
$(VENV):
	@echo "→ Creating virtualenv with '$$(which $(PYTHON))' ..."
	@$(PYTHON) -m venv $(VENV)

install: $(VENV)
	@echo "→ Installing dependencies..."
	@$(VENV_PYTHON) -m pip install --upgrade pip setuptools wheel
	@$(VENV_PYTHON) -m pip install -e ".[dev]"

bootstrap: $(VENV) install-git-hooks
.PHONY: bootstrap

# Cleanup
clean:
	@$(MAKE) clean-soft
	@echo "→ Cleaning (.venv) ..."
	@$(RM) $(VENV)

clean-soft:
	@echo "→ Cleaning (no .venv) ..."
	@$(RM) \
	  .pytest_cache htmlcov coverage.xml dist build *.egg-info .tox demo .tmp_home \
	  .ruff_cache .mypy_cache .hypothesis .coverage.* .coverage .benchmarks \
	  spec.json openapitools.json node_modules .mutmut-cache session.sqlite site \
	  docs/reference artifacts usage_test usage_test_artifacts citation.bib .cache || true
	@find . -type d -name '__pycache__' -exec $(RM) {} +

# Pipelines
all: clean install test lint quality security api docs build sbom citation
	@echo "✔ All targets completed"

# Run independent checks in parallel
lint quality security api docs: | bootstrap
.NOTPARALLEL:

dev-cli-status:
	@cargo run -q -p bijux-cli-bin -- dev cli status --text

dev-cli-crate-health:
	@cargo run -q -p bijux-cli-bin -- dev cli crate-health --text

dev-cli-parity:
	@cargo run -q -p bijux-cli-bin -- dev cli parity --text

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
  print(tomllib.load(open("pyproject.toml","rb"))["project"]["version"])' \
  2>/dev/null || echo 0.0.0 \
))
endef

help:
	@awk 'BEGIN{FS=":.*##"; OFS="";} \
	  /^##@/ {gsub(/^##@ */,""); print "\n\033[1m" $$0 "\033[0m"; next} \
	  /^[a-zA-Z0-9_.-]+:.*##/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' \
	  $(MAKEFILE_LIST)
.PHONY: help

##@ Core
clean: ## Remove virtualenv, caches, build, and artifacts
clean-soft: ## Remove build artifacts but keep .venv
install: ## Install project in editable mode into .venv
bootstrap: ## Setup environment & install git hooks
all: ## Run full pipeline (clean → citation)
dev-cli-status: ## Show maintainer status report via bijux dev cli
dev-cli-crate-health: ## Show crate health and duplication report via bijux dev cli
dev-cli-parity: ## Show parity summary via bijux dev cli
help: ## Show this help
