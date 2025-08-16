# Citation Configuration (ephemeral venv per run)

# Directories & files
CFFENV              := artifacts/.cffenv
CFFENV_PY           := $(CFFENV)/bin/python
CFFCONVERT_BIN      := $(CFFENV)/bin/cffconvert

CITATION_FILE       := CITATION.cff
CITATION_DIR        := artifacts/citation
CITATION_BIB        := $(CITATION_DIR)/citation.bib
CITATION_RIS        := $(CITATION_DIR)/citation.ris
CITATION_ENDNOTE    := $(CITATION_DIR)/citation.enw

.PHONY: citation citation-install citation-validate citation-bibtex citation-ris citation-endnote citation-check citation-clean-env citation-clean

# Orchestrator: one install → all steps → clean venv → check
citation: | $(CITATION_DIR)
	@echo "→ Generating citation artifacts (ephemeral env)"
	@$(MAKE) -s citation-install
	@$(MAKE) -s NO_CLEAN=1 citation-validate
	@$(MAKE) -s NO_CLEAN=1 citation-bibtex
	@$(MAKE) -s NO_CLEAN=1 citation-ris
	@$(MAKE) -s NO_CLEAN=1 citation-endnote
	@$(MAKE) -s citation-clean-env
	@$(MAKE) -s citation-check
	@echo "✔ Citation artifacts generated in '$(CITATION_DIR)'"

citation-install:
	@echo "→ Installing cffconvert into isolated env @ $(CFFENV)"
	@python3 -m venv "$(CFFENV)"
	@$(CFFENV_PY) -m pip install --upgrade pip
	@$(CFFENV_PY) -m pip install --upgrade "cffconvert>=2.0"

citation-validate:
	@if [ ! -f "$(CITATION_FILE)" ]; then echo "✘ $(CITATION_FILE) not found"; exit 1; fi
	@if [ ! -x "$(CFFCONVERT_BIN)" ]; then $(MAKE) -s citation-install; fi
	@echo "→ Validating $(CITATION_FILE)"
	@$(CFFCONVERT_BIN) --validate --infile "$(CITATION_FILE)"
	@# Ephemeral cleanup when called directly (not via orchestrator)
	@if [ -z "$(NO_CLEAN)" ]; then $(MAKE) -s citation-clean-env; fi

citation-bibtex: | $(CITATION_DIR)
	@if [ ! -x "$(CFFCONVERT_BIN)" ]; then $(MAKE) -s citation-install; fi
	@$(CFFCONVERT_BIN) -f bibtex --infile "$(CITATION_FILE)" --outfile "$(CITATION_BIB)"
	@echo "  ✔ BibTeX -> $(CITATION_BIB)"
	@sed -i.bak '1s/@misc{[^,]*/@misc{Mousavi2025-bijux-cli/' "$(CITATION_BIB)" && rm -f "$(CITATION_BIB).bak"
	@if [ -z "$(NO_CLEAN)" ]; then $(MAKE) -s citation-clean-env; fi

citation-ris: | $(CITATION_DIR)
	@if [ ! -x "$(CFFCONVERT_BIN)" ]; then $(MAKE) -s citation-install; fi
	@$(CFFCONVERT_BIN) -f ris --infile "$(CITATION_FILE)" --outfile "$(CITATION_RIS)"
	@echo "  ✔ RIS -> $(CITATION_RIS)"
	@if [ -z "$(NO_CLEAN)" ]; then $(MAKE) -s citation-clean-env; fi

citation-endnote: | $(CITATION_DIR)
	@if [ ! -x "$(CFFCONVERT_BIN)" ]; then $(MAKE) -s citation-install; fi
	@$(CFFCONVERT_BIN) -f endnote --infile "$(CITATION_FILE)" --outfile "$(CITATION_ENDNOTE)"
	@echo "  ✔ EndNote -> $(CITATION_ENDNOTE)"
	@if [ -z "$(NO_CLEAN)" ]; then $(MAKE) -s citation-clean-env; fi

citation-check:
	@test -s "$(CITATION_BIB)" && test -s "$(CITATION_RIS)" && test -s "$(CITATION_ENDNOTE)" \
		|| { echo "✘ Empty citation artifact detected"; exit 1; }

citation-clean-env:
	@echo "→ Removing citation tool env @ $(CFFENV)"
	@rm -rf "$(CFFENV)"

citation-clean:
	@echo "→ Cleaning citation artifacts & tool env"
	@rm -rf "$(CITATION_DIR)" "$(CFFENV)"

$(CITATION_DIR):
	@mkdir -p "$(CITATION_DIR)"

##@ Citation
citation: ## Validate CITATION.cff and generate all citation formats (BibTeX, RIS, EndNote) using an ephemeral venv
citation-install: ## Create isolated venv & install cffconvert (usually called automatically)
citation-validate: ## Validate CITATION.cff file structure & compliance
citation-bibtex: ## Generate BibTeX from CITATION.cff
citation-ris: ## Generate RIS from CITATION.cff
citation-endnote: ## Generate EndNote from CITATION.cff
citation-check: ## Ensure generated citation artifacts are valid & non-empty
citation-clean-env: ## Remove the temporary cffconvert environment only
citation-clean: ## Remove citation artifacts and the tool environment
