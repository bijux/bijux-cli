CONTRACT_SHELL_SCRIPTS := \
	makes/bin/run_core_rust_gate.sh \
	makes/bin/run_file_processing_demo.sh

.PHONY: contract-tests

contract-tests: docs-publication-check check-badges ## Validate Core repository contracts
	@command -v shellcheck >/dev/null 2>&1 || { echo "shellcheck is required" >&2; exit 1; }
	@shellcheck $(CONTRACT_SHELL_SCRIPTS)
