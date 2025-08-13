# Build Configuration

# Directories & files
BUILD_DIR ?= build

.PHONY: build build-clean

build:
	@echo "→ Preparing Python package artifacts..."
	@mkdir -p $(BUILD_DIR)
	@echo "→ Building wheel + sdist..."
	@$(VENV_PYTHON) -m build --wheel --sdist --outdir $(BUILD_DIR) .
	@echo "✔ Build artifacts ready in '$(BUILD_DIR)'"

build-clean:
	@echo "→ Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR) dist *.egg-info
	@find . -type d -name "__pycache__" -exec rm -rf {} +
	@echo "✔ Build artifacts cleaned"

##@ Build
build: ## Build both wheel and source distribution into $(BUILD_DIR)
build-clean: ## Remove all build artifacts (wheel, sdist, egg-info, caches)
