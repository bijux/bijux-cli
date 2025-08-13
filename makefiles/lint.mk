# Lint Configuration


INTERROGATE_PATHS ?= src/bijux_cli

RUFF        := $(ACT)/ruff
MYPY        := $(ACT)/mypy
PYTYPE      := $(ACT)/pytype
CODESPELL   := $(ACT)/codespell
PYRIGHT     := $(ACT)/pyright
PYDOCSTYLE  := $(ACT)/pydocstyle
RADON       := $(ACT)/radon
INTERROGATE := $(ACT)/interrogate

.PHONY: lint lint-file lint-dir interrogate-report

lint:
	@echo "→ Running all linting checks"
	@$(MAKE) lint-dir dir=src/bijux_cli
	@$(MAKE) lint-dir dir=tests
	@echo "✔ Linting completed successfully"

lint-file:
ifndef file
	$(error Usage: make lint-file file=path/to/file.py)
endif
	@$(call run_tool,RuffFormat,$(RUFF) format)
	@$(call run_tool,Ruff,$(RUFF) check --fix --config config/ruff.toml)
	@$(call run_tool,Mypy,$(MYPY) --config-file config/mypy.ini --strict)
	@$(call run_tool,Codespell,$(CODESPELL) -I config/bijux.dic)
	@$(call run_tool,Pyright,$(PYRIGHT) --project config/pyrightconfig.json)
	@$(call run_tool,Radon,$(RADON) cc -s -a)
	@$(call run_tool,Pydocstyle,$(PYDOCSTYLE) --convention=google)

lint-dir:
ifndef dir
	$(error Usage: make lint-dir dir=<directory_path>)
endif
	@echo "=== Linting directory '$(dir)' ==="
	@for file in $$(find $(dir) -type f -name '*.py'); do \
		$(MAKE) lint-file file=$$file; \
	done
	@if $(VENV_PYTHON) -c 'import sys; sys.exit(0) if sys.version_info >= (3, 13) else sys.exit(1)'; then \
        echo "→ Skipping Pytype (unsupported on Python > 3.12)"; \
    else \
        $(call run_tool,Pytype,$(PYTYPE) --disable import-error); \
    fi

interrogate-report:
	@echo "→ Generating docstring coverage report (<100%)"
	@set +e; \
	  OUT="$$( $(INTERROGATE) --no-color --verbose $(INTERROGATE_PATHS) )"; \
	  rc=$$?; \
	  OFF="$$(printf '%s\n' "$$OUT" | awk -F'|' 'NR>3 && $$0 ~ /^\|/ { \
	    name=$$2; cov=$$6; gsub(/^[ \t]+|[ \t]+$$/, "", name); gsub(/^[ \t]+|[ \t]+$$/, "", cov); \
	    if (name !~ /^-+$$/ && cov != "100%") printf("  - %s (%s)\n", name, cov); \
	  }')"; \
	  if [ -n "$$OFF" ]; then printf "%s\n" "$$OFF"; else echo "✔ All files 100% documented"; fi; \
	  exit $$rc

##@ Lint
lint: ## Run all lint checks (ruff, mypy, pyright, codespell, radon, pydocstyle)
lint-file: ## Lint a single Python file (requires file=<path>)
lint-dir: ## Lint all Python files in a directory (requires dir=<path>)
interrogate-report: ## Generate docstring coverage report for files <100%
