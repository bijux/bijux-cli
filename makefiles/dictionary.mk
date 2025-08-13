# Dictionary Configuration

DICTIONARY_FILE := config/bijux.dic
DICTIONARY_TMP  := $(DICTIONARY_FILE).tmp

.PHONY: dictionary dictionary-clean dictionary-check

dictionary:
	@echo "→ Normalizing dictionary: $(DICTIONARY_FILE)"
	@tr '[:upper:]' '[:lower:]' < "$(DICTIONARY_FILE)" | sort -u > "$(DICTIONARY_TMP)"
	@mv "$(DICTIONARY_TMP)" "$(DICTIONARY_FILE)"
	@echo "✔ Dictionary normalized & sorted"

dictionary-check:
	@if ! diff -q <(tr '[:upper:]' '[:lower:]' < "$(DICTIONARY_FILE)" | sort -u) "$(DICTIONARY_FILE)" >/dev/null; then \
		echo "✘ Dictionary requires normalization (run: make dictionary)"; \
		exit 1; \
	else \
		echo "✔ Dictionary is normalized & sorted"; \
	fi

dictionary-clean:
	@rm -f "$(DICTIONARY_TMP)"
	@echo "→ Dictionary temp files cleaned"

##@ Dictionary
dictionary: ## Normalize (lowercase), sort, and deduplicate the custom dictionary
dictionary-check: ## Validate that dictionary is already normalized & sorted
dictionary-clean: ## Remove temporary dictionary artifacts
