# SBOM Configuration (pip-audit → CycloneDX JSON)

PACKAGE_NAME        ?= bijux-cli
GIT_SHA             ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo unknown)

GIT_TAG_EXACT       := $(shell git describe --tags --exact-match 2>/dev/null | sed -E 's/^v//')
GIT_TAG_LATEST      := $(shell git describe --tags --abbrev=0 2>/dev/null | sed -E 's/^v//')

PYPROJECT_VERSION    = $(call read_pyproject_version)

PKG_VERSION         ?= $(if $(GIT_TAG_EXACT),$(GIT_TAG_EXACT),\
                           $(if $(PYPROJECT_VERSION),$(PYPROJECT_VERSION),\
                           $(if $(GIT_TAG_LATEST),$(GIT_TAG_LATEST),0.0.0)))

GIT_DESCRIBE        := $(shell git describe --tags --long --dirty --always 2>/dev/null)
PKG_VERSION_FULL    := $(if $(GIT_TAG_EXACT),$(PKG_VERSION),\
                          $(shell echo "$(GIT_DESCRIBE)" \
                            | sed -E 's/^v//; s/-([0-9]+)-g([0-9a-f]+)(-dirty)?$$/+\\1.g\\2\\3/'))

SBOM_VERSION        := $(if $(PKG_VERSION_FULL),$(PKG_VERSION_FULL),$(PKG_VERSION))

SBOM_DIR            ?= artifacts/sbom
SBOM_PROD_REQ       ?= requirements/prod.txt
SBOM_DEV_REQ        ?= requirements/dev.txt
SBOM_FORMAT         ?= cyclonedx-json            # pip-audit format
SBOM_CLI            ?= cyclonedx                 # cyclonedx-cli for validation
SBOM_IGNORE_IDS     ?= PYSEC-2022-42969
SBOM_IGNORE_FLAGS    = $(foreach V,$(SBOM_IGNORE_IDS),--ignore-vuln $(V))

PIP_AUDIT           := $(if $(ACT),$(ACT)/pip-audit,pip-audit)
PIP_AUDIT_FLAGS      = --progress-spinner off --format $(SBOM_FORMAT)

SBOM_PROD_FILE      := $(SBOM_DIR)/$(PACKAGE_NAME)-$(SBOM_VERSION)-$(GIT_SHA).prod.cdx.json
SBOM_DEV_FILE       := $(SBOM_DIR)/$(PACKAGE_NAME)-$(SBOM_VERSION)-$(GIT_SHA).dev.cdx.json

.PHONY: sbom sbom-prod sbom-dev sbom-validate sbom-summary sbom-clean

sbom: sbom-clean sbom-prod sbom-dev sbom-summary
	@echo "✔ SBOMs generated in $(SBOM_DIR)"

sbom-prod:
	@mkdir -p "$(SBOM_DIR)"
	@if [ -s "$(SBOM_PROD_REQ)" ]; then \
	  echo "→ SBOM (prod via $(SBOM_PROD_REQ))"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    -r "$(SBOM_PROD_REQ)" --output "$(SBOM_PROD_FILE)" || true; \
	else \
	  echo "→ SBOM (prod fallback: current venv)"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    --output "$(SBOM_PROD_FILE)" || true; \
	fi

sbom-dev:
	@mkdir -p "$(SBOM_DIR)"
	@if [ -s "$(SBOM_DEV_REQ)" ]; then \
	  echo "→ SBOM (dev via $(SBOM_DEV_REQ))"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    -r "$(SBOM_DEV_REQ)" --output "$(SBOM_DEV_FILE)" || true; \
	else \
	  echo "→ SBOM (dev fallback: current venv)"; \
	  $(PIP_AUDIT) $(PIP_AUDIT_FLAGS) $(SBOM_IGNORE_FLAGS) \
	    --output "$(SBOM_DEV_FILE)" || true; \
	fi

sbom-validate:
	@if [ -z "$(SBOM_CLI)" ]; then echo "✘ SBOM_CLI not set"; exit 1; fi
	@command -v $(SBOM_CLI) >/dev/null 2>&1 || { echo "✘ '$(SBOM_CLI)' not found. Install it or set SBOM_CLI."; exit 1; }
	@if ! find "$(SBOM_DIR)" -maxdepth 1 -name '*.cdx.json' -print -quit | grep -q .; then \
	  echo "✘ No SBOM files in $(SBOM_DIR)"; exit 1; \
	fi
	@for f in "$(SBOM_DIR)"/*.cdx.json; do \
	  echo "→ Validating $$f"; \
	  $(SBOM_CLI) validate --input-format json --input-file "$$f"; \
	done

sbom-summary:
	@mkdir -p "$(SBOM_DIR)"
	@if ! find "$(SBOM_DIR)" -maxdepth 1 -name '*.cdx.json' -print -quit | grep -q .; then \
	  echo "→ No SBOM files found in $(SBOM_DIR); skipping summary"; \
	  exit 0; \
	fi
	@echo "→ Writing SBOM summary"
	@summary="$(SBOM_DIR)/summary.txt"; : > "$$summary"; \
	if command -v jq >/dev/null 2>&1; then \
	  for f in "$(SBOM_DIR)"/*.cdx.json; do \
	    comps=$$(jq -r '(.components|length) // 0' "$$f"); \
	    echo "$$(basename "$$f")  components=$$comps" >> "$$summary"; \
	  done; \
	else \
	  tmp="$(SBOM_DIR)/_sbom_summary.py"; \
	  echo "import glob, json, os"                                  >  "$$tmp"; \
	  echo "sbom_dir = r'$(SBOM_DIR)'"                              >> "$$tmp"; \
	  echo "for f in glob.glob(os.path.join(sbom_dir, '*.cdx.json')):" >> "$$tmp"; \
	  echo "    try:"                                               >> "$$tmp"; \
	  echo "        with open(f, 'r', encoding='utf-8') as fh:"     >> "$$tmp"; \
	  echo "            d = json.load(fh)"                          >> "$$tmp"; \
	  echo "        comps = len(d.get('components', []) or [])"     >> "$$tmp"; \
	  echo "    except Exception:"                                  >> "$$tmp"; \
	  echo "        comps = '?'"                                    >> "$$tmp"; \
	  echo "    print(os.path.basename(f) + '  components=' + str(comps))" >> "$$tmp"; \
	  python3 "$$tmp" >> "$$summary" || true; \
	  rm -f "$$tmp"; \
	fi; \
	sed -n '1,5p' "$$summary" 2>/dev/null || true

sbom-clean:
	@echo "→ Cleaning SBOM artifacts"
	@mkdir -p "$(SBOM_DIR)"
	@rm -f \
	  "$(SBOM_DIR)/$(PACKAGE_NAME)-0.0.0-"*.cdx.json \
	  "$(SBOM_DIR)/$(PACKAGE_NAME)--"*.cdx.json || true

##@ SBOM
sbom:           ## Generate SBOMs for prod/dev (pip-audit → CycloneDX JSON) and a short summary
sbom-validate:  ## Validate all generated SBOMs with CycloneDX CLI
sbom-summary:   ## Write a brief components summary to $(SBOM_DIR)/summary.txt (best-effort)
sbom-clean:     ## Remove stale SBOM artifacts from $(SBOM_DIR)
