# Citation Configuration

# Directories & files
CFFENV              := artifacts/.cffenv
CFFENV_PY           := $(CFFENV)/bin/python
CFFCONVERT_ISO      := $(CFFENV)/bin/cffconvert

CITATION_FILE       := CITATION.cff
CITATION_DIR        := artifacts/citation
CITATION_BIB        := $(CITATION_DIR)/citation.bib
CITATION_RIS        := $(CITATION_DIR)/citation.ris
CITATION_ENDNOTE    := $(CITATION_DIR)/citation.enw

# Tools
CFFCONVERT          := $(ACT)/cffconvert

.PHONY: citation citation-install citation-validate citation-bibtex citation-ris citation-endnote citation-check citation-clean

# Orchestrator
citation: citation-install citation-validate citation-bibtex citation-ris citation-endnote citation-check
	@echo "✔ Citation artifacts generated in '$(CITATION_DIR)'"

citation-install:
	@echo "→ Installing cffconvert into isolated env @ $(CFFENV)"
	@python3 -m venv "$(CFFENV)"
	@$(CFFENV_PY) -m pip install --upgrade pip
	@$(CFFENV_PY) -m pip install --upgrade "cffconvert>=2.0"

citation-validate:
	@if [ ! -f "$(CITATION_FILE)" ]; then echo "✘ $(CITATION_FILE) not found"; exit 1; fi
	@echo "→ Validating $(CITATION_FILE)"
	@$(CFFCONVERT_ISO) --validate --infile "$(CITATION_FILE)"

citation-bibtex: | $(CITATION_DIR)
	@$(CFFCONVERT_ISO) -f bibtex --infile "$(CITATION_FILE)" --outfile "$(CITATION_BIB)"
	@echo "  ✔ BibTeX -> $(CITATION_BIB)"
	@sed -i.bak '1s/@misc{[^,]*/@misc{Mousavi2025-bijux-cli/' "$(CITATION_BIB)" && rm -f "$(CITATION_BIB).bak"

citation-ris: | $(CITATION_DIR)
	@$(CFFCONVERT_ISO) -f ris --infile "$(CITATION_FILE)" --outfile "$(CITATION_RIS)"
	@echo "  ✔ RIS -> $(CITATION_RIS)"

citation-endnote: | $(CITATION_DIR)
	@$(CFFCONVERT_ISO) -f endnote --infile "$(CITATION_FILE)" --outfile "$(CITATION_ENDNOTE)"
	@echo "  ✔ EndNote -> $(CITATION_ENDNOTE)"

citation-check:
	@test -s "$(CITATION_BIB)" && test -s "$(CITATION_RIS)" && test -s "$(CITATION_ENDNOTE)" \
		|| { echo "✘ Empty citation artifact detected"; exit 1; }

citation-clean:
	@echo "→ Cleaning citation artifacts & tool env"
	@rm -rf "$(CITATION_DIR)" "$(CFFENV)"

$(CITATION_DIR):
	@mkdir -p "$(CITATION_DIR)"


##@ Citation
citation: ## Validate CITATION.cff and generate all citation formats (BibTeX, RIS, EndNote)
citation-install: ## Create isolated venv & install cffconvert
citation-validate: ## Validate CITATION.cff file structure & compliance
citation-bibtex: ## Generate BibTeX from CITATION.cff
citation-ris: ## Generate RIS from CITATION.cff
citation-endnote: ## Generate EndNote from CITATION.cff
citation-check: ## Ensure generated citation artifacts are valid & non-empty
citation-clean: ## Remove citation artifacts and cffconvert environment
