# API configuration

# Directories & files
API_HOST            ?= 127.0.0.1
API_PORT            ?= 8000
API_BASE_PATH       ?= /v1
API_APP             ?= app
API_MODULE          ?= bijux_cli.httpapi
API_WAIT_SECS       ?= 30
API_LOG             ?= artifacts/api-server.log
API_FACTORY         ?=
SCHEMA_URL          ?= http://$(API_HOST):$(API_PORT)
HEALTH_PATH         ?= /health

NODE_MODULES        := ./node_modules
ALL_API_SCHEMAS     := $(shell find api -type f \( -name '*.yaml' -o -name '*.yml' \))
OPENAPI_GENERATOR_VERSION ?= 7.14.0

ifneq ($(strip $(API_FACTORY)),)
API_CMD ?= $(VENV_PYTHON) -c 'import importlib, uvicorn; \
    mod=importlib.import_module("$(API_MODULE)"); \
    app=getattr(mod, "$(API_FACTORY)")(); \
    uvicorn.run(app, host="$(API_HOST)", port=$(API_PORT))'
else
API_CMD ?= $(VENV_PYTHON) -m uvicorn $(API_MODULE):$(API_APP) \
    --host $(API_HOST) --port $(API_PORT)
endif

# Toolchain paths
PRANCE                 := $(ACT)/prance
OPENAPI_SPEC_VALIDATOR := $(ACT)/openapi-spec-validator
REDOCLY                := $(NODE_MODULES)/.bin/redocly
OPENAPI_GENERATOR      := $(NODE_MODULES)/.bin/openapi-generator-cli
SCHEMATHESIS           := $(ACT)/schemathesis
SCHEMATHESIS_OPTS     ?= --checks=all

# Macro: Schema validation pipeline
define validate-schema
	@echo "→ Validating schema: $(1)"
	@$(PRANCE) validate "$(1)"
	@$(OPENAPI_SPEC_VALIDATOR) "$(1)"
	@$(REDOCLY) lint "$(1)"
	@NODE_NO_WARNINGS=1 $(OPENAPI_GENERATOR) validate -i "$(1)"
endef

.PHONY: api api-install api-lint api-test node_deps node_bootstrap

## Run full API validation (install → lint → test)
api: api-install api-lint api-test

api-install: | $(VENV) node_deps
	@echo "→ Installing API toolchain..."
	@command -v npm  >/dev/null || { echo "✘ npm not found"; exit 1; }
	@command -v curl >/dev/null || { echo "✘ curl not found"; exit 1; }
	@command -v java >/dev/null || { echo "✘ java not found"; exit 1; }
	@$(VENV_PYTHON) -m pip install --quiet prance openapi-spec-validator uvicorn schemathesis

api-lint: | node_deps
	@if [ -z "$(ALL_API_SCHEMAS)" ]; then \
		echo "✘ No API schemas found under api/*.yaml"; exit 1; \
	fi
	@echo "→ Linting OpenAPI specs..."
	$(foreach schema,$(ALL_API_SCHEMAS),$(call validate-schema,$(schema)))
	@echo "✔ API schemas valid."

api-test: | $(VENV) node_deps
	@if [ -z "$(ALL_API_SCHEMAS)" ]; then \
		echo "✘ No API schemas found under api/*.yaml"; exit 1; \
	fi
	@echo "→ Starting API server: $(API_CMD)"
	@mkdir -p $(dir $(API_LOG))
	@set -euo pipefail; \
		$(API_CMD) >"$(API_LOG)" 2>&1 & PID=$$!; \
		trap "kill $$PID >/dev/null 2>&1 || true; wait $$PID >/dev/null 2>&1 || true" EXIT; \
		echo "→ Waiting up to $(API_WAIT_SECS)s for readiness..."; \
		for i in $$(seq 1 $(API_WAIT_SECS)); do \
			if curl -fsS '$(SCHEMA_URL)$(HEALTH_PATH)' >/dev/null 2>&1; then break; fi; \
			sleep 1; \
			if ! kill -0 $$PID >/dev/null 2>&1; then \
				echo "✘ API crashed — see $(API_LOG)"; exit 1; \
			fi; \
		done
	@echo "→ Schemathesis tests..."
	@for schema in $(ALL_API_SCHEMAS); do \
		$(SCHEMATHESIS) run "$$schema" --base-url "$(SCHEMA_URL)$(API_BASE_PATH)" $(SCHEMATHESIS_OPTS) || exit $$?; \
	done

node_deps: node_modules/.deps-ok

node_modules/.deps-ok: package.json
	@command -v npm >/dev/null || { echo "✘ npm not found"; exit 1; }
	@if [ -f package-lock.json ]; then \
		echo "→ Using npm ci"; npm ci --silent; \
	else \
		echo "→ Using npm install"; npm install --silent; \
	fi
	@echo "→ Pinning OpenAPI Generator CLI $(OPENAPI_GENERATOR_VERSION)"
	@npx --yes @openapitools/openapi-generator-cli@latest version-manager set $(OPENAPI_GENERATOR_VERSION)
	@touch $@

node_bootstrap:
	@command -v npm >/dev/null || { echo "✘ npm not found"; exit 1; }
	@[ -f package.json ] || npm init -y >/dev/null
	@npm install --save-dev --save-exact @redocly/cli @openapitools/openapi-generator-cli
	@$(MAKE) node_deps


##@ API
api: ## Run full API validation workflow (install → lint → test with Schemathesis)
api-install: ## Install full API toolchain (Python deps + Node deps + validation CLIs)
api-lint: ## Lint & validate all OpenAPI specifications under api/*.yaml
api-test: ## Start API server, verify readiness, fuzz-test endpoints with Schemathesis
