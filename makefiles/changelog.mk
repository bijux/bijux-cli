# Changelog Configuration

TC_ENSURE  := command -v $(TOWNCRIER) >/dev/null 2>&1 || $(VENV_PYTHON) -m pip install -q "towncrier>=23,<25"
TC_BASE    := $(shell bash -c 'b=$${TC_BASE:-origin/main}; git rev-parse --verify --quiet $$b >/dev/null || b=HEAD~1; echo $$b')
TC_PYVER   := import tomllib; print(tomllib.load(open("pyproject.toml","rb"))["project"]["version"])
TOWNCRIER  := $(ACT)/towncrier

.PHONY: changelog changelog-check changelog-preview generate-fragment

changelog: | $(VENV) generate-fragment
	@grep -q "<!-- towncrier start -->" CHANGELOG.md || { echo "✘ Missing '<!-- towncrier start -->' in CHANGELOG.md"; exit 1; }
	@$(TC_ENSURE)
	@VER=$$($(VENV_PYTHON) -c '$(TC_PYVER)'); \
	  test -n "$$VER" -a "$$VER" != "0.0.0" || { echo "✘ Could not resolve version from pyproject.toml"; exit 1; }; \
	  if grep -qE "## \[$$VER\]" CHANGELOG.md; then \
	    echo "✘ Version $$VER already exists in CHANGELOG.md — bump version in pyproject.toml"; exit 1; \
	  fi; \
	  PREVIEW=$$($(TOWNCRIER) build --config pyproject.toml --draft --version "$$VER"); \
	  if echo "$$PREVIEW" | grep -q "No significant changes."; then \
	    echo "✔ No significant changes — changelog skipped"; exit 0; \
	  fi; \
	  $(TOWNCRIER) build --config pyproject.toml --version "$$VER" --yes

changelog-check: | $(VENV)
	@$(TC_ENSURE)
	@$(TOWNCRIER) check --config pyproject.toml --compare-with="$(TC_BASE)"

changelog-preview: | $(VENV)
	@$(TC_ENSURE)
	@VER=$$($(VENV_PYTHON) -c '$(TC_PYVER)'); \
	  test -n "$$VER" -a "$$VER" != "0.0.0" || { echo "✘ Could not resolve version from pyproject.toml"; exit 1; }; \
	  $(TOWNCRIER) build --config pyproject.toml --draft --version "$$VER"

generate-fragment:
	@if [ ! -d changelog.d ] || ! find changelog.d -type f 2>/dev/null | grep -q .; then \
	  msg="$$(git log -1 --pretty=%B | tr -d '\r')"; \
	  if ! echo "$$msg" | grep -q ':'; then \
	    echo "✘ Commit message must be in format 'type: description' to auto-generate fragment"; exit 1; \
	  fi; \
	  type="$$(echo "$$msg" | cut -d: -f1 | tr -d '[:space:]')"; \
	  desc="$$(echo "$$msg" | cut -d: -f2- | sed 's/^[[:space:]]*//')"; \
	  case "$$type" in \
	    feat) kind=feature ;; \
	    fix)  kind=bugfix  ;; \
	    refactor|perf|docs|chore|test) kind=misc ;; \
	    *) kind=misc ;; \
	  esac; \
	  id=$$(date +%s); \
	  mkdir -p changelog.d; \
	  printf '%s\n' "$$desc" > "changelog.d/$$id.$$kind.md"; \
	  echo "✔ Generated changelog.d/$$id.$$kind.md from commit message"; \
	fi

##@ Changelog
changelog: ## Generate & append changelog from towncrier fragments
changelog-check: ## Check if changelog fragments are present & valid
changelog-preview: ## Preview upcoming changelog without writing
