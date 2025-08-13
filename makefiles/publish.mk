# Publish Configuration

DIST_DIR         ?= build
TWINE_REPOSITORY ?= pypi
PYPI_INDEX       ?= https://pypi.org/simple
TESTPYPI_INDEX   ?= https://test.pypi.org/simple

TWINE            := $(VENV_PYTHON) -m twine

.PHONY: publish publish-test twine twine-check twine-upload twine-upload-test verify-test-install check-version

twine: publish

publish:
	@echo "→ Publishing to PyPI"
	@$(MAKE) check-version
	@$(MAKE) build
	@$(MAKE) twine-check
	@$(MAKE) twine-upload
	@echo "✔ Published to PyPI"

publish-test:
	@echo "→ Publishing to TestPyPI"
	@$(MAKE) check-version
	@$(MAKE) build
	@$(MAKE) twine-check
	@$(MAKE) twine-upload-test
	@echo "✔ Published to TestPyPI"

twine-check:
	@if ls $(DIST_DIR)/bijux_cli-$(PKG_VERSION)* >/dev/null 2>&1; then :; \
	else echo "✘ No artifacts for $(PKG_VERSION) in '$(DIST_DIR)'. Run 'make build' first."; exit 1; fi
	@echo "→ Validating artifacts for $(PKG_VERSION)"
	@$(TWINE) check $(DIST_DIR)/bijux_cli-$(PKG_VERSION)*

twine-upload:
	@echo "→ Uploading $(PKG_VERSION) to repository '$(TWINE_REPOSITORY)'"
	@$(TWINE) upload --non-interactive --skip-existing $(DIST_DIR)/bijux_cli-$(PKG_VERSION)* -r $(TWINE_REPOSITORY)

twine-upload-test:
	@$(MAKE) twine-upload TWINE_REPOSITORY=testpypi

verify-test-install:
	@echo "→ Verifying installation from TestPyPI"
	@tmp=$$(mktemp -d); \
	python3 -m venv $$tmp/venv; \
	$$tmp/venv/bin/pip install -U pip; \
	$$tmp/venv/bin/pip install -i $(TESTPYPI_INDEX) --extra-index-url $(PYPI_INDEX) bijux-cli==$(PKG_VERSION); \
	$$tmp/venv/bin/bijux --version; \
	echo "✔ Installed successfully from TestPyPI"; \
	echo "Temp venv at $$tmp (delete when done)"

check-version:
	@[ "$(PKG_VERSION)" != "0.0.0" ] || { echo "✘ PKG_VERSION resolved to 0.0.0"; exit 1; }

##@ Publish
twine: ## Alias to publish
publish: ## Upload release to PyPI (build → validate → upload)
publish-test: ## Upload release to TestPyPI (build → validate → upload)
verify-test-install: ## Install from TestPyPI into temp venv and run CLI
