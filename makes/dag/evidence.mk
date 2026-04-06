SHELL := /bin/bash

evidence-all: evidence-verify ## Run the canonical evidence verification entrypoint

evidence-verify: ## Run the full evidence verification suite
	$(call run_or_fail,Run evidence foundation verification,$(DEV_TOOL) verify evidence-foundation)

evidence-battle: ## Run battle evidence verification
	$(call run_or_fail,Run battle evidence verification,$(DEV_TOOL) verify evidence-battle)

evidence-authoring: ## Run authoring evidence verification
	$(call run_or_fail,Run authoring evidence verification,$(DEV_TOOL) verify evidence-authoring)

evidence-cache: ## Run cache evidence verification
	$(call run_or_fail,Run cache evidence verification,$(DEV_TOOL) verify evidence-cache)

evidence-replay: ## Run replay evidence verification
	$(call run_or_fail,Run replay evidence verification,$(DEV_TOOL) verify evidence-replay)

evidence-compat: ## Run compatibility evidence verification
	$(call run_or_fail,Run compatibility evidence verification,$(DEV_TOOL) verify evidence-compat)

evidence-fault: ## Run fault evidence verification
	$(call run_or_fail,Run fault evidence verification,$(DEV_TOOL) verify evidence-fault)

evidence-perf: ## Run performance evidence verification
	$(call run_or_fail,Run performance evidence verification,$(DEV_TOOL) verify evidence-perf)

evidence-compare: ## Run comparison evidence verification
	$(call run_or_fail,Run comparison evidence verification,$(DEV_TOOL) verify evidence-compare)

evidence-schema: ## Run evidence schema verification
	$(call run_or_fail,Run evidence schema verification,$(DEV_TOOL) verify evidence-schema)

evidence-registry: ## Run evidence registry verification
	$(call run_or_fail,Run evidence registry verification,$(DEV_TOOL) verify evidence-registry)

evidence-consumers: ## Run evidence consumer verification
	$(call run_or_fail,Run evidence consumer verification,$(DEV_TOOL) verify evidence-consumers)

evidence-release-set: ## Run release evidence set verification
	$(call run_or_fail,Run release evidence set verification,$(DEV_TOOL) verify evidence-release-set)

evidence-report: ## Generate evidence suite summary reports
	$(call run_or_fail,Generate evidence summary reports,$(DEV_TOOL) repo evidence-summary-report)

evidence-summary: evidence-report ## Compatibility alias for evidence-report

evidence-clean: ## Clean generated evidence artifacts
	$(call run_or_fail,Clean generated evidence artifacts,$(DEV_TOOL) artifacts-clean)

.PHONY: evidence-all evidence-verify evidence-battle evidence-authoring evidence-cache evidence-replay
.PHONY: evidence-compat evidence-fault evidence-perf evidence-compare evidence-schema evidence-registry
.PHONY: evidence-consumers evidence-release-set evidence-report evidence-summary evidence-clean
