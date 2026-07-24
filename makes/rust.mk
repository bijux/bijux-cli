# Rust quality checks and reports that write only under `artifacts/`.

RS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT_ABS)/rust
RS_RUN_ID ?= $(RUN_ID)

RS_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/target)
RS_COVERAGE_DIR ?= $(RS_ARTIFACT_ROOT)/coverage/$(RS_RUN_ID)
RS_LCOV_FILE ?= $(RS_COVERAGE_DIR)/lcov.info
RS_RELEASE_VALIDATION_DIR ?= $(RS_ARTIFACT_ROOT)/release-validation/$(RS_RUN_ID)
RS_RELEASE_TREE_DIR ?= $(abspath $(RS_RELEASE_VALIDATION_DIR)/workspace)
RS_RELEASE_CARGO_CONFIG ?= $(RS_RELEASE_TREE_DIR)/.cargo/config.toml
RS_RELEASE_VALIDATION_TARGET_DIR ?= $(abspath $(RS_RELEASE_VALIDATION_DIR)/target)
RS_RELEASE_TREE_VERSION_FILE ?= $(RS_RELEASE_VALIDATION_DIR)/workspace-version.txt
RS_RELEASE_FMT_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/fmt.txt
RS_RELEASE_CLIPPY_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/clippy.txt
RS_RELEASE_TEST_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/test.txt
RS_RELEASE_DOC_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/doc.txt
RS_RELEASE_PACKAGE_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/package.txt
RS_RELEASE_PUBLISH_DRY_RUN_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/publish-dry-run.txt
RS_RELEASE_SMOKE_REPORT ?= $(RS_RELEASE_VALIDATION_DIR)/smoke.txt
RS_RELEASE_CARGO_JOBS ?= 2
RS_DEV_CLI_BIN ?= $(RS_TARGET_DIR)/debug/bijux-dev-cli
RS_DAG_BIN ?= $(RS_TARGET_DIR)/debug/bijux-dag
RS_RELEASE_DAG_BIN ?= $(RS_RELEASE_VALIDATION_TARGET_DIR)/debug/bijux-dag
RS_RELEASE_BUNDLE_DIR ?= $(RS_ARTIFACT_ROOT)/build
DAG_RELEASE_PACKAGE ?= bijux-dag-cli
DAG_RELEASE_BIN ?= bijux-dag
RS_BUILD_GIT_SHA ?= $(shell git rev-parse --short HEAD 2>/dev/null || true)
RS_BUILD_GIT_SHA_ENV ?= $(if $(strip $(RS_BUILD_GIT_SHA)),BIJUX_DAG_BUILD_GIT_SHA="$(strip $(RS_BUILD_GIT_SHA))")
RUST_PUBLIC_DAG_PACKAGES ?= bijux-dag-core bijux-dag-artifacts bijux-dag-runtime bijux-dag-app bijux-dag-cli
RUST_PUBLISH_PACKAGES ?= bijux-dag-core bijux-dag-artifacts bijux-dag-runtime bijux-dag-app bijux-dag-cli bijux-cli
RUST_PUBLISH_DRY_RUN ?= 1
RUST_PUBLISH_SKIP_EXISTING ?= 1
RUST_PUBLISH_ALLOW_DIRTY ?= 0
RUST_PUBLISH_REGISTRY ?= crates-io

CARGO_TERM_PROGRESS_WHEN ?= always
CARGO_TERM_PROGRESS_WIDTH ?= 120
CARGO_TERM_VERBOSE ?= false
CARGO_TERM_COLOR ?= always

NEXTEST_PROFILE ?= default
NEXTEST_RELEASE_PROFILE ?= ci
NEXTEST_FULL_PROFILE ?= ci
NEXTEST_PROFILE_FAST ?= $(NEXTEST_PROFILE)
NEXTEST_PROFILE_SLOW ?= $(NEXTEST_PROFILE)
NEXTEST_PROFILE_ALL ?= $(NEXTEST_FULL_PROFILE)
NEXTEST_SLOW_NAME_EXPR ?= test(/^slow__/)
NEXTEST_STATUS_LEVEL ?= all
NEXTEST_FINAL_STATUS_LEVEL ?= all
CORE_RUST_GATE_BIN ?= makes/bin/run_core_rust_gate.sh
RUST_GATE_BIN ?= $(CORE_RUST_GATE_BIN)
RUST_AUDIT_PREREQUISITES += audit-policy-rs

.PHONY: test-release-rs prepare-release-tree-rs fmt-release-rs clippy-release-rs
.PHONY: test-release-workspace-rs doc-release-rs package-release-rs
.PHONY: publish-dry-run-release-rs smoke-release-rs release-validate-rs
.PHONY: coverage audit-policy-rs publish-rs build-dag-release-bundle
.NOTPARALLEL: prepare-release-tree-rs fmt-release-rs clippy-release-rs test-release-workspace-rs doc-release-rs package-release-rs publish-dry-run-release-rs smoke-release-rs release-validate-rs

##@ Rust
test-release-rs: ## Run the required Rust release-candidate lane
	@NEXTEST_PROFILE_FAST="$(NEXTEST_RELEASE_PROFILE)" "$(RUST_GATE_BIN)" test

prepare-release-tree-rs: ## Prepare a clean release-candidate tree from committed HEAD
	@mkdir -p "$(RS_RELEASE_VALIDATION_DIR)"
	@rm -rf "$(RS_RELEASE_TREE_DIR)"
	@release_version="$$(python3 -c "import tomllib; from pathlib import Path; print(tomllib.load(Path('Cargo.toml').open('rb'))['workspace']['package']['version'])")"; \
	printf '%s\n' "$$release_version" > "$(RS_RELEASE_TREE_VERSION_FILE)"; \
	printf '%s\n' "prepare: release tree version $$release_version"; \
	python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$(RS_RELEASE_TREE_DIR)" --version "$$release_version" >/dev/null
	@mkdir -p "$(dir $(RS_RELEASE_CARGO_CONFIG))"
	# Patch staged public DAG crates into the clean release tree so dry-run publish verifies topological public dependencies before the new release is present on crates.io.
	@printf '\n[patch.crates-io]\nbijux-dag-core = { path = "crates/bijux-dag-core" }\nbijux-dag-artifacts = { path = "crates/bijux-dag-artifacts" }\nbijux-dag-runtime = { path = "crates/bijux-dag-runtime" }\nbijux-dag-app = { path = "crates/bijux-dag-app" }\nbijux-dag-cli = { path = "crates/bijux-dag-cli" }\n' >> "$(RS_RELEASE_CARGO_CONFIG)"

fmt-release-rs: prepare-release-tree-rs ## Run release-candidate formatting validation in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_FMT_REPORT))"
	@printf '%s\n' "run: cargo fmt --all -- --check"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo fmt --all -- --check 2>&1 | tee "$(abspath $(RS_RELEASE_FMT_REPORT))"

clippy-release-rs: prepare-release-tree-rs ## Run release-candidate clippy validation in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_CLIPPY_REPORT))"
	@printf '%s\n' "run: cargo clippy --workspace --all-targets --all-features --locked -- -D warnings"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	CLIPPY_CONF_DIR="configs/rust" \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings 2>&1 | tee "$(abspath $(RS_RELEASE_CLIPPY_REPORT))"

test-release-workspace-rs: prepare-release-tree-rs ## Run release-candidate workspace tests in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_TEST_REPORT))"
	@printf '%s\n' "run: cargo test --workspace --all-targets --all-features --locked"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" cargo build -p bijux-dag-cli --bin bijux-dag; \
	$(RS_BUILD_GIT_SHA_ENV) \
	BIJUX_DAG_BIN="$(RS_RELEASE_DAG_BIN)" \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo test --workspace --all-targets --all-features --locked 2>&1 | tee "$(abspath $(RS_RELEASE_TEST_REPORT))"

doc-release-rs: prepare-release-tree-rs ## Run release-candidate docs build in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_DOC_REPORT))"
	@printf '%s\n' "run: cargo doc --workspace --all-features --no-deps"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo doc --workspace --all-features --no-deps 2>&1 | tee "$(abspath $(RS_RELEASE_DOC_REPORT))"

package-release-rs: prepare-release-tree-rs ## Run release-candidate cargo package listings for public DAG crates
	@mkdir -p "$(dir $(RS_RELEASE_PACKAGE_REPORT))"
	@rm -f "$(RS_RELEASE_PACKAGE_REPORT)"
	@set -euo pipefail; \
	for package in $(RUST_PUBLIC_DAG_PACKAGES); do \
		printf '%s\n' "run: cargo package -p $${package} --list" | tee -a "$(RS_RELEASE_PACKAGE_REPORT)"; \
		( \
			cd "$(RS_RELEASE_TREE_DIR)"; \
			$(RS_BUILD_GIT_SHA_ENV) \
			CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
			CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" \
			CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
			CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
			CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
			CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
			cargo package -p "$${package}" --list \
		) 2>&1 | tee -a "$(RS_RELEASE_PACKAGE_REPORT)"; \
	done

publish-dry-run-release-rs: prepare-release-tree-rs ## Run release-candidate cargo publish dry-runs for public DAG crates
	@mkdir -p "$(dir $(RS_RELEASE_PUBLISH_DRY_RUN_REPORT))"
	@rm -f "$(RS_RELEASE_PUBLISH_DRY_RUN_REPORT)"
	@set -euo pipefail; \
	for package in $(RUST_PUBLIC_DAG_PACKAGES); do \
		printf '%s\n' "run: cargo publish -p $${package} --dry-run --locked" | tee -a "$(RS_RELEASE_PUBLISH_DRY_RUN_REPORT)"; \
		( \
			cd "$(RS_RELEASE_TREE_DIR)"; \
			$(RS_BUILD_GIT_SHA_ENV) \
			CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
			CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" \
			CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
			CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
			CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
			CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
			cargo publish -p "$${package}" --dry-run --locked \
		) 2>&1 | tee -a "$(RS_RELEASE_PUBLISH_DRY_RUN_REPORT)"; \
	done

smoke-release-rs: prepare-release-tree-rs ## Run release-candidate DAG CLI smoke tests in a clean tree
	@mkdir -p "$(dir $(RS_RELEASE_SMOKE_REPORT))"
	@printf '%s\n' "run: cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture"
	@set -o pipefail; \
	cd "$(RS_RELEASE_TREE_DIR)"; \
	$(RS_BUILD_GIT_SHA_ENV) \
	CARGO_TARGET_DIR="$(RS_RELEASE_VALIDATION_TARGET_DIR)" \
	CARGO_BUILD_JOBS="$(RS_RELEASE_CARGO_JOBS)" \
	CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" \
	CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" \
	CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" \
	CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" \
	cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture 2>&1 | tee "$(abspath $(RS_RELEASE_SMOKE_REPORT))"

release-validate-rs: fmt-release-rs clippy-release-rs test-release-workspace-rs doc-release-rs package-release-rs publish-dry-run-release-rs smoke-release-rs ## Run the canonical Rust release validation suite

coverage: coverage-rs ## Run coverage and refresh tracked coverage reports
	@mkdir -p artifacts/coverage
	@cp "$(RS_LCOV_FILE)" artifacts/coverage/lcov.info
	@BIJUX_COVERAGE_LCOV_PATH="$(RS_LCOV_FILE)" cargo run --locked -p bijux-dev --bin generate_line_coverage_reports

audit-policy-rs: ## Verify Core security policy before Cargo advisory checks
	@mkdir -p "$(RS_TARGET_DIR)"
	@CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
		cargo run --locked -q -p bijux-dev --bin bijux-dev-dag -- security

publish-rs: ## Publish Rust crates and dry-run by default
	@set -euo pipefail; \
	if [ -z "$(RUST_PUBLISH_PACKAGES)" ]; then \
		echo "RUST_PUBLISH_PACKAGES is empty; nothing to publish"; \
		exit 1; \
	fi; \
	dry_run_flag=""; \
	if [ "$(RUST_PUBLISH_DRY_RUN)" = "1" ]; then \
		dry_run_flag="--dry-run"; \
	fi; \
	allow_dirty_flag=""; \
	if [ "$(RUST_PUBLISH_ALLOW_DIRTY)" = "1" ]; then \
		allow_dirty_flag="--allow-dirty"; \
	fi; \
	workspace_root="."; \
	temp_root=""; \
	if [ -n "$(RELEASE_VERSION)" ]; then \
		temp_root="$$(mktemp -d "$${TMPDIR:-/tmp}/bijux-release-tree.XXXXXX")"; \
		trap 'test -n "$${temp_root}" && rm -rf "$${temp_root}"' EXIT; \
		python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$${temp_root}" --version "$(RELEASE_VERSION)" >/dev/null; \
		workspace_root="$${temp_root}"; \
		echo "→ Publishing from release tree stamped to $(RELEASE_VERSION)"; \
	elif [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ]; then \
		workspace_version="$$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p['\''name'\'']: p['\''version'\''] for p in data['\''packages'\'']}; print(pkgs.get('\''bijux-cli'\'', '\'''\''))' 2>/dev/null)"; \
		case "$${workspace_version}" in \
			*-*) if [ "$(PUBLISH_ALLOW_PRERELEASE)" != "1" ]; then \
				echo "Refusing to publish prerelease workspace version $${workspace_version} without RELEASE_VERSION or PUBLISH_ALLOW_PRERELEASE=1"; \
				exit 1; \
			fi ;; \
		esac; \
	fi; \
	for pkg in $(RUST_PUBLISH_PACKAGES); do \
		publish_version="$$(cargo metadata --manifest-path "$${workspace_root}/Cargo.toml" --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p["name"]: p["version"] for p in data["packages"]}; print(pkgs.get(sys.argv[1], ""))' "$$pkg" 2>/dev/null)"; \
		if [ -z "$${publish_version}" ]; then \
			echo "Could not resolve version for package $$pkg from cargo metadata"; \
			exit 1; \
		fi; \
		if [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ] && [ "$(RUST_PUBLISH_SKIP_EXISTING)" = "1" ]; then \
			status="$$(curl -s -o /dev/null -w '%{http_code}' "https://crates.io/api/v1/crates/$$pkg/$${publish_version}" || true)"; \
			if [ "$${status}" = "200" ]; then \
				echo "→ Skipping $$pkg $${publish_version}; already present on crates.io"; \
				continue; \
			fi; \
		fi; \
		echo "→ cargo publish -p $$pkg@$${publish_version} --registry $(RUST_PUBLISH_REGISTRY) $$dry_run_flag"; \
		publish_log="$$(mktemp "$${TMPDIR:-/tmp}/bijux-cargo-publish.XXXXXX.log")"; \
		if $(RS_BUILD_GIT_SHA_ENV) CARGO_TARGET_DIR="$(RS_TARGET_DIR)" \
			cargo publish \
				--locked \
				--manifest-path "$${workspace_root}/Cargo.toml" \
				--registry "$(RUST_PUBLISH_REGISTRY)" \
				-p "$$pkg" \
				$$allow_dirty_flag \
				$$dry_run_flag >"$${publish_log}" 2>&1; then \
			cat "$${publish_log}"; \
			rm -f "$${publish_log}"; \
		else \
			cat "$${publish_log}"; \
			if [ "$(RUST_PUBLISH_DRY_RUN)" != "1" ] && [ "$(RUST_PUBLISH_SKIP_EXISTING)" = "1" ] && \
				grep -Eq 'already exists on crates\.io index|already uploaded' "$${publish_log}"; then \
				echo "→ Skipping $$pkg $${publish_version}; registry already has this release"; \
				rm -f "$${publish_log}"; \
				continue; \
			fi; \
			rm -f "$${publish_log}"; \
			exit 1; \
		fi; \
	done

build-dag-release-bundle: ## Build a stamped bijux-dag binary release bundle under artifacts/rust/build
	@mkdir -p "$(RS_RELEASE_BUNDLE_DIR)"
	@set -euo pipefail; \
	workspace_root="."; \
	temp_root=""; \
	if [ -n "$(RELEASE_VERSION)" ]; then \
		temp_root="$$(mktemp -d "$${TMPDIR:-/tmp}/bijux-release-tree.XXXXXX")"; \
		trap 'test -n "$${temp_root}" && rm -rf "$${temp_root}"' EXIT; \
		python3 "$(RELEASE_TREE_SCRIPT)" --workspace-root . --output-dir "$${temp_root}" --version "$(RELEASE_VERSION)" >/dev/null; \
		workspace_root="$${temp_root}"; \
		echo "→ Building DAG release bundle from release tree stamped to $(RELEASE_VERSION)"; \
	fi; \
	host_triple="$$(rustc -vV | awk '/^host:/ {print $$2}')"; \
	bundle_version="$$(cargo metadata --manifest-path "$${workspace_root}/Cargo.toml" --no-deps --format-version 1 | python3 -c 'import json,sys; data=json.load(sys.stdin); pkgs={p["name"]: p["version"] for p in data["packages"]}; print(pkgs.get(sys.argv[1], ""))' "$(DAG_RELEASE_PACKAGE)" 2>/dev/null)"; \
	if [ -z "$${bundle_version}" ]; then \
		echo "Could not resolve version for package $(DAG_RELEASE_PACKAGE) from cargo metadata"; \
		exit 1; \
	fi; \
	stage_dir="$(RS_RELEASE_BUNDLE_DIR)/$(DAG_RELEASE_BIN)-bundle"; \
	archive_name="$(DAG_RELEASE_BIN)-v$${bundle_version}-$${host_triple}.tar.gz"; \
	archive_path="$(RS_RELEASE_BUNDLE_DIR)/$${archive_name}"; \
	rm -rf "$${stage_dir}"; \
	rm -f "$${archive_path}" "$${archive_path}.sha256"; \
	mkdir -p "$${stage_dir}/bin"; \
	$(RS_BUILD_GIT_SHA_ENV) CARGO_TARGET_DIR="$(RS_TARGET_DIR)" cargo build --release --locked --manifest-path "$${workspace_root}/Cargo.toml" -p "$(DAG_RELEASE_PACKAGE)" --bin "$(DAG_RELEASE_BIN)"; \
	cp "$(RS_TARGET_DIR)/release/$(DAG_RELEASE_BIN)" "$${stage_dir}/bin/$(DAG_RELEASE_BIN)"; \
	cp "$${workspace_root}/LICENSE" "$${stage_dir}/LICENSE"; \
	cp "$${workspace_root}/crates/$(DAG_RELEASE_PACKAGE)/README.md" "$${stage_dir}/README.md"; \
	printf 'version=%s\ncrate=%s\nbinary=%s\nhost_triple=%s\n' "$${bundle_version}" "$(DAG_RELEASE_PACKAGE)" "$(DAG_RELEASE_BIN)" "$${host_triple}" > "$${stage_dir}/release-metadata.txt"; \
	printf '%s\n' \
		'Install by extracting this archive and placing `bin/$(DAG_RELEASE_BIN)` on your PATH.' \
		'For source publication and API documentation, use the published `$(DAG_RELEASE_PACKAGE)` crate.' \
		> "$${stage_dir}/INSTALL.txt"; \
	( \
		cd "$${stage_dir}"; \
		find . -type f ! -name 'checksums.txt' -print | LC_ALL=C sort | while IFS= read -r file_path; do \
			shasum -a 256 "$${file_path}"; \
		done > checksums.txt; \
	); \
	tar -C "$${stage_dir}" -czf "$${archive_path}" .; \
	shasum -a 256 "$${archive_path}" > "$${archive_path}.sha256"; \
	echo "→ Built DAG release bundle: $${archive_path}"
