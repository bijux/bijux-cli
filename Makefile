# SPDX-License-Identifier: MIT
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
	@echo "→ Cleaning..."
	@$(RM) \
	  $(VENV) \
	  .pytest_cache htmlcov coverage.xml dist build *.egg-info .tox demo .tmp_home \
	  .ruff_cache .mypy_cache .pytype .hypothesis .coverage.* .coverage .benchmarks \
	  spec.json openapitools.json node_modules .mutmut-cache session.sqlite site \
	  docs/reference artifacts usage_test usage_test_artifacts citation.bib || true
	@if [ "$(OS)" != "Windows_NT" ]; then \
	  find . -type d -name '__pycache__' -exec $(RM) {} +; \
	fi

clean-soft:
	@echo "→ Cleaning (no .venv) ..."
	@$(RM) \
	  .pytest_cache htmlcov coverage.xml dist build *.egg-info .tox demo .tmp_home \
	  .ruff_cache .mypy_cache .pytype .hypothesis .coverage.* .coverage .benchmarks \
	  spec.json openapitools.json node_modules .mutmut-cache session.sqlite site \
	  docs/reference artifacts usage_test usage_test_artifacts citation.bib || true
	@if [ "$(OS)" != "Windows_NT" ]; then \
	  find . -type d -name '__pycache__' -exec $(RM) {} +; \
	fi

# Pipelines
all: clean install test lint quality security api docs build sbom citation
	@echo "✔ All targets completed"

# Run independent checks in parallel
lint quality security api docs: | bootstrap
.NOTPARALLEL:

all-parallel: clean install
	@$(MAKE) -j4 quality security api docs
	@$(MAKE) build sbom citation
	@echo "✔ All targets completed (parallel mode)"

# Pre-push Gate - Pre-Commit!
pre-push:
	@$(PYTEST) -q -m "not e2e and not slow"
	@$(MAKE) quality
	@$(MAKE) security
	@$(MAKE) api
	@$(MAKE) docs
	@$(MAKE) changelog-check
	@echo "✔ pre-push gate passed"
.PHONY: pre-push

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
all-parallel: ## Run pipeline with parallelized lint, quality, security, api, and docs
pre-push: ## Run pre-push gate: tests, quality, security, API, docs, changelog-check
help: ## Show this help
