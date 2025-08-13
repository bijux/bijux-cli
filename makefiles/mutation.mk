# Mutation Configuration

MUTMUT     := $(ACT)/mutmut
COSMIC_RAY := $(ACT)/cosmic-ray

.PHONY: mutation mutation-clean mutation-cosmic mutation-mutmut

mutation:
	@echo "→ Running full mutation testing suite"
	@$(MAKE) mutation-clean
	@$(MAKE) mutation-cosmic
	@$(MAKE) mutation-mutmut
	@echo "✔ Mutation testing completed"

mutation-clean:
	@echo "→ Cleaning mutation test artifacts"
	@$(RM) session.sqlite .mutmut-cache

mutation-cosmic:
	@echo "→ [Cosmic-Ray] Initializing session"
	@$(COSMIC_RAY) init config/cosmic-ray.toml session.sqlite
	@echo "→ [Cosmic-Ray] Executing mutation tests"
	@$(COSMIC_RAY) exec config/cosmic-ray.toml session.sqlite
	@echo "→ [Cosmic-Ray] Generating report"
	@$(COSMIC_RAY) report config/cosmic-ray.toml session.sqlite

mutation-mutmut:
	@echo "→ [Mutmut] Running mutation tests"
	@$(MUTMUT) run

##@ Mutation
mutation: ## Run all mutation tests (Cosmic-Ray + Mutmut)
mutation-clean: ## Remove mutation testing artifacts (session.sqlite, .mutmut-cache)
mutation-cosmic: ## Run mutation testing with Cosmic-Ray
mutation-mutmut: ## Run mutation testing with Mutmut
