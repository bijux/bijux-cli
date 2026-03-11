# Lint Configuration (no root cache pollution)

RUFF        := $(ACT)/ruff
MYPY        := $(ACT)/mypy
CODESPELL   := $(ACT)/codespell
PYDOCSTYLE  := $(ACT)/pydocstyle
RADON       := $(ACT)/radon

# Targets & dirs
LINT_DIRS           ?= crates/bijux-cli-python/python/bijux_cli_py crates/bijux-cli-python/tests/python
LINT_ARTIFACTS_DIR  ?= artifacts/lint

# Tool caches inside artifacts_pages/lint
RUFF_CACHE_DIR      ?= $(LINT_ARTIFACTS_DIR)/.ruff_cache
MYPY_CACHE_DIR      ?= $(LINT_ARTIFACTS_DIR)/.mypy_cache

# In case these are not defined elsewhere
VENV_PYTHON         ?= python3

.PHONY: fmt fmt-check lint lint-artifacts lint-file lint-dir lint-clean

fmt: | $(VENV)
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -euo pipefail; { \
	  echo "→ Ruff format"; \
	  $(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS); \
	} 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"

fmt-check: | $(VENV)
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)"
	@set -euo pipefail; { \
	  echo "→ Ruff format (check)"; \
	  $(RUFF) format --check --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS); \
	} 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"

lint: lint-artifacts
	@echo "✔ Linting completed (logs in '$(LINT_ARTIFACTS_DIR)')"

lint-artifacts: | $(VENV)
	@mkdir -p "$(LINT_ARTIFACTS_DIR)" "$(RUFF_CACHE_DIR)" "$(MYPY_CACHE_DIR)"
	@set -euo pipefail; { \
	  echo "→ Ruff format (check)"; \
	  $(RUFF) format --check --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS); \
	} 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff-format.log"
	@set -euo pipefail; $(RUFF) check --fix --config configs/ruff.toml --cache-dir "$(RUFF_CACHE_DIR)" $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/ruff.log"
	@set -euo pipefail; $(MYPY) --config-file configs/mypy.ini --strict --cache-dir "$(MYPY_CACHE_DIR)" $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/mypy.log"
	@set -euo pipefail; $(CODESPELL) -I configs/bijux.dic $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/codespell.log"
	@set -euo pipefail; $(RADON) cc -s -a $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/radon.log"
	@set -euo pipefail; $(PYDOCSTYLE) --convention=google $(LINT_DIRS) 2>&1 | tee "$(LINT_ARTIFACTS_DIR)/pydocstyle.log"
	@[ -d .mypy_cache ] && echo "→ removing stray .mypy_cache" && rm -rf .mypy_cache || true
	@[ -d .ruff_cache ] && echo "→ removing stray .ruff_cache" && rm -rf .ruff_cache || true
	@printf "OK\n" > "$(LINT_ARTIFACTS_DIR)/_passed"

lint-file:
ifndef file
	$(error Usage: make lint-file file=path/to/file.py)
endif
	@$(call run_tool,RuffFormat,$(RUFF) format --cache-dir "$(RUFF_CACHE_DIR)")
	@$(call run_tool,Ruff,$(RUFF) check --fix --config configs/ruff.toml --cache-dir "$(RUFF_CACHE_DIR)")
	@$(call run_tool,Mypy,$(MYPY) --config-file configs/mypy.ini --strict --cache-dir "$(MYPY_CACHE_DIR)")
	@$(call run_tool,Codespell,$(CODESPELL) -I configs/bijux.dic)
	@$(call run_tool,Radon,$(RADON) cc -s -a)
	@$(call run_tool,Pydocstyle,$(PYDOCSTYLE) --convention=google)

lint-dir:
ifndef dir
	$(error Usage: make lint-dir dir=<directory_path>)
endif
	@$(MAKE) LINT_DIRS="$(dir)" lint-artifacts

lint-clean:
	@echo "→ Cleaning lint artifacts"
	@rm -rf "$(LINT_ARTIFACTS_DIR)" .mypy_cache .ruff_cache || true
	@echo "✔ done"

##@ Lint
lint: ## Run all lint checks; save logs to artifacts_pages/lint/ (ruff/mypy caches under artifacts_pages/lint)
fmt: ## Auto-format codebase using Ruff
fmt-check: ## Check formatting with Ruff (no changes)
lint-artifacts: ## Same as 'lint' (explicit), generates logs
lint-file: ## Lint a single file (requires file=<path>)
lint-dir: ## Lint a directory (requires dir=<path>)
lint-clean: ## Remove lint artifacts_pages, including caches
