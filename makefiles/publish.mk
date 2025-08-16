# Publish Configuration (build → validate → upload)

DIST_DIR            ?= artifacts/build
PKG_DIST_NAME       ?= bijux_cli
PY                  ?= python3
TWINE               ?= $(PY) -m twine
TWINE_REPOSITORY    ?= pypi
TWINE_USERNAME      ?= __token__
TWINE_PASSWORD      ?= $(PYPI_API_TOKEN)
SKIP_TWINE_CHECK    ?= 0
SKIP_EXISTING       ?= 1


PKG_VERSION ?= $(shell \
  echo 'import importlib, importlib.util, pathlib; \
tl = importlib.import_module("tomllib") if importlib.util.find_spec("tomllib") else importlib.import_module("tomli"); \
print(tl.loads(pathlib.Path("pyproject.toml").read_bytes())["project"]["version"])' \
  | $(PY) - 2>/dev/null || echo 0.0.0 \
)


.PHONY: publish publish-test twine twine-check twine-upload twine-upload-test ensure-dists check-version verify-test-install

twine: publish  ## alias

publish: check-version build twine-check twine-upload
	@echo "✔ Published $(PKG_DIST_NAME) $(PKG_VERSION) to $(TWINE_REPOSITORY)"

publish-test: check-version build twine-check twine-upload-test
	@echo "✔ Published $(PKG_DIST_NAME) $(PKG_VERSION) to testpypi"

check-version:
	@echo "→ Package version: $(PKG_VERSION)"
	@[ "$(PKG_VERSION)" != "0.0.0" ] || { echo "✘ PKG_VERSION resolved to 0.0.0"; exit 1; }

ensure-dists:
	@echo "→ Verifying artifacts for $(PKG_VERSION) in '$(DIST_DIR)'"
	@test -d "$(DIST_DIR)" || { echo "✘ Dist dir missing: $(DIST_DIR)"; exit 1; }
	@whl=$$(ls "$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-"*.whl 2>/dev/null | head -n1); \
	 sdist="$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION).tar.gz"; \
	 test -n "$$whl" || { echo "✘ Missing wheel: $(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-*.whl"; exit 1; }; \
	 test -f "$$sdist" || { echo "✘ Missing sdist: $$sdist"; exit 1; }; \
	 ls -lh "$$whl" "$$sdist"

twine-check: ensure-dists
ifeq ($(SKIP_TWINE_CHECK),1)
	@echo "→ Skipping twine check (SKIP_TWINE_CHECK=$(SKIP_TWINE_CHECK))"
else
	@echo "→ Running twine check"
	@whl=$$(ls "$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-"*.whl | head -n1); \
	 sdist="$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION).tar.gz"; \
	 $(TWINE) check "$$whl" "$$sdist"
endif

twine-upload: ensure-dists
	@echo "→ Uploading $(PKG_DIST_NAME) $(PKG_VERSION) to repository '$(TWINE_REPOSITORY)'"
	@test -n "$(TWINE_PASSWORD)" || { echo "✘ PYPI_API_TOKEN (TWINE_PASSWORD) not set"; exit 1; }
	@whl=$$(ls "$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION)-"*.whl | head -n1); \
	 sdist="$(DIST_DIR)/$(PKG_DIST_NAME)-$(PKG_VERSION).tar.gz"; \
	 SKIP=""; [ "$(SKIP_EXISTING)" = "1" ] && SKIP="--skip-existing"; \
	 $(TWINE) upload --non-interactive --disable-progress-bar $$SKIP \
	   --repository "$(TWINE_REPOSITORY)" -u "$(TWINE_USERNAME)" -p "$(TWINE_PASSWORD)" \
	   "$$whl" "$$sdist"

twine-upload-test:
	@$(MAKE) twine-upload TWINE_REPOSITORY=testpypi

verify-test-install:
	@echo "→ Verifying installation from TestPyPI"
	@tmp=$$(mktemp -d); \
	$(PY) -m venv $$tmp/venv; \
	$$tmp/venv/bin/pip install -U pip; \
	$$tmp/venv/bin/pip install -i https://test.pypi.org/simple --extra-index-url https://pypi.org/simple bijux-cli==$(PKG_VERSION); \
	$$tmp/venv/bin/bijux --version; \
	echo "✔ TestPyPI install OK"; \
	echo "Temp venv at $$tmp (delete when done)"

##@ Publish
publish:             ## Upload release to PyPI (build → validate → upload)
publish-test:        ## Upload release to TestPyPI (build → validate → upload)
verify-test-install: ## Install from TestPyPI into temp venv and run CLI
