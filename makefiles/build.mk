# Build Configuration — keep outputs under artifacts_pages/

# Dirs & flags
BUILD_DIR        ?= artifacts/build
CHECK_DISTS      ?= 1             # set to 0 to skip twine check

# Absolute paths (safer if a target changes CWD)
BUILD_DIR_ABS    := $(abspath $(BUILD_DIR))
PYPROJECT_ABS    := $(abspath pyproject.toml)

.PHONY: build build-sdist build-wheel build-check build-tools build-clean

build-tools: | $(VENV)
	@echo "→ Ensuring build toolchain..."
	@$(VENV_PYTHON) -m pip install -U pip
	@$(VENV_PYTHON) -m pip install --upgrade build twine

build: build-tools
	@if [ ! -f "$(PYPROJECT_ABS)" ]; then echo "✘ pyproject.toml not found"; exit 1; fi
	@echo "→ Preparing Python package artifacts..."
	@mkdir -p "$(BUILD_DIR_ABS)"
	@echo "→ Building wheel + sdist → $(BUILD_DIR_ABS)"
	@$(VENV_PYTHON) -m build --wheel --sdist --outdir "$(BUILD_DIR_ABS)" .
	@if [ "$(CHECK_DISTS)" = "1" ]; then \
	  echo "→ Validating distributions with twine"; \
	  $(VENV_PYTHON) -m twine check "$(BUILD_DIR_ABS)"/* 2>&1 | tee "$(BUILD_DIR_ABS)/twine-check.log"; \
	else \
	  echo "→ Skipping twine check (CHECK_DISTS=$(CHECK_DISTS))"; \
	fi
	@echo "✔ Build artifacts ready in '$(BUILD_DIR_ABS)'"
	@ls -l "$(BUILD_DIR_ABS)" || true

build-sdist: build-tools
	@if [ ! -f "$(PYPROJECT_ABS)" ]; then echo "✘ pyproject.toml not found"; exit 1; fi
	@mkdir -p "$(BUILD_DIR_ABS)"
	@echo "→ Building sdist → $(BUILD_DIR_ABS)"
	@$(VENV_PYTHON) -m build --sdist --outdir "$(BUILD_DIR_ABS)" .

build-wheel: build-tools
	@if [ ! -f "$(PYPROJECT_ABS)" ]; then echo "✘ pyproject.toml not found"; exit 1; fi
	@mkdir -p "$(BUILD_DIR_ABS)"
	@echo "→ Building wheel → $(BUILD_DIR_ABS)"
	@$(VENV_PYTHON) -m build --wheel --outdir "$(BUILD_DIR_ABS)" .

build-check:
	@if ls "$(BUILD_DIR_ABS)"/* 1>/dev/null 2>&1; then \
	  $(VENV_PYTHON) -m twine check "$(BUILD_DIR_ABS)"/* 2>&1 | tee "$(BUILD_DIR_ABS)/twine-check.log"; \
	else \
	  echo "✘ No artifacts in $(BUILD_DIR_ABS) to check"; exit 1; \
	fi

build-clean:
	@echo "→ Cleaning build artifacts..."
	@rm -rf "$(BUILD_DIR_ABS)" || true
	@rm -rf build dist *.egg-info || true
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@echo "✔ Build artifacts cleaned"

##@ Build
build-tools: ## Ensure local venv has build tooling (pip, build, twine)
build-clean: ## Remove all build artifacts_pages (artifacts_pages/build + legacy build/, dist/, *.egg-info)
build: ## Build both wheel and source distribution into artifacts_pages/build (twine check optional via CHECK_DISTS=1)
build-sdist: ## Build sdist only into artifacts_pages/build
build-wheel: ## Build wheel only into artifacts_pages/build
build-check: ## Run twine check on artifacts_pages/build/*
