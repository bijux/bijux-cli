use super::*;

pub(super) const CHECK_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "fmt",
        description: "cargo fmt check",
        domain: "style",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["fmt", "--all", "--", "--check"]),
    },
    SuiteDef {
        id: "lint",
        description: "cargo clippy with warnings as errors",
        domain: "quality",
        slow: true,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_status("cargo", &["fmt", "--all", "--", "--check"])?;
            run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        },
    },
    SuiteDef {
        id: "security",
        description: "cargo audit policy check",
        domain: "supply-chain",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_audit_allowlist_quality_gate()?;
            run_deny_policy_deviations_gate()?;
            run_status("cargo", &["audit"])
        },
    },
    SuiteDef {
        id: "dep-guard",
        description: "forbidden dependency reference check",
        domain: "policy",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_dep_guard(),
    },
];

pub(super) const TEST_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "unit",
        description: "cargo test --workspace",
        domain: "runtime",
        slow: true,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "--workspace"]),
    },
    SuiteDef {
        id: "arch",
        description: "repository architecture tests",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "-p", "bijux-dev"]),
    },
    SuiteDef {
        id: "e2e-matrix",
        description: "end-to-end matrix against binary and crate entrypoints",
        domain: "e2e",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_e2e_matrix(),
    },
    SuiteDef {
        id: "evidence-consumer-integrity",
        description: "tests and fixtures consume evidence-owned scenario assets",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_evidence_consumers_verify(),
    },
];

pub(super) const CONTRACT_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "compat",
        description: "core compat fixture assertions",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || {
            run_status(
                "cargo",
                &["run", "-p", "bijux-dag-cli", "--bin", "bijux-dag", "--", "compat"],
            )
        },
    },
    SuiteDef {
        id: "golden",
        description: "run/replay golden execution parity",
        domain: "runtime",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_golden(),
    },
    SuiteDef {
        id: "public-api",
        description: "public API surface contract",
        domain: "quality",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_public_api(),
    },
    SuiteDef {
        id: "validation-rules-doc",
        description: "core validation rule IDs are documented",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_validation_rule_docs_guard(),
    },
    SuiteDef {
        id: "schema-contracts",
        description: "schema source files and fixtures are present and versioned",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_schema_contracts_guard(),
    },
    SuiteDef {
        id: "adapter-conformance",
        description: "runtime adapter descriptor conformance checks",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_status(
                "cargo",
                &[
                    "test",
                    "-p",
                    "bijux-dag-runtime",
                    "adapter_descriptor_requires_identity_and_schema_version",
                ],
            )
        },
    },
    SuiteDef {
        id: "backend-conformance",
        description: "runtime execution backend conformance checks",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_status("cargo", &["test", "-p", "bijux-dag-runtime", "execution_backend_contract"])
        },
    },
    SuiteDef {
        id: "evidence-consumer-integrity",
        description: "evidence ownership, drift, and consumer references are enforceable",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_evidence_ownership_verify()?;
            run_evidence_drift_verify()?;
            run_evidence_consumers_verify()
        },
    },
];

pub(super) const DOC_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "handbook-indexes",
        description: "check canonical public handbook index files",
        domain: "docs",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            let root = repo_root()?;
            let required = [
                "docs/index.md",
                "docs/bijux-core/index.md",
                "docs/bijux-cli/index.md",
                "docs/bijux-dag/index.md",
                "docs/bijux-dev/index.md",
            ];
            let missing =
                required.into_iter().filter(|path| !root.join(path).is_file()).collect::<Vec<_>>();
            if !missing.is_empty() {
                return Err(format!("missing canonical handbook indexes: {}", missing.join(", ")));
            }
            Ok(())
        },
    },
    SuiteDef {
        id: "guarantee-evidence",
        description: "guarantee claims require proof references",
        domain: "docs",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_guarantee_guard(),
    },
    SuiteDef {
        id: "governance-lint",
        description: "docs metadata completeness and topic/orphan lint",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || super::docs_governance::run_docs_governance_lint(),
    },
];

pub(super) const RELEASE_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "verify",
        description: "canonical release validation plus readiness evidence",
        domain: "release",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_release_verify(),
    },
    SuiteDef {
        id: "readiness",
        description: "release readiness evidence aggregation",
        domain: "release",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_release_readiness_report(),
    },
    SuiteDef {
        id: "compatibility-matrix",
        description: "generate compatibility matrix from supported fixtures",
        domain: "release",
        slow: false,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_release_compatibility_matrix(),
    },
    SuiteDef {
        id: "post-release-verify",
        description: "run minimal installed-binary workflow",
        domain: "release",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_post_release_verify(None),
    },
    SuiteDef {
        id: "reproducibility-check",
        description: "verify release tag reproducibility against current commit",
        domain: "release",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || Ok(()),
    },
    SuiteDef {
        id: "evidence-bundle",
        description: "write release evidence bundle",
        domain: "release",
        slow: false,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_release_evidence_bundle(None),
    },
];

include!("suite_catalog_repo.rs");

#[cfg(test)]
mod tests {
    use super::*;

    fn suite_ids(suites: &[SuiteDef]) -> Vec<&'static str> {
        suites.iter().map(|suite| suite.id).collect()
    }

    #[test]
    fn check_suites_include_style_quality_and_policy() {
        let ids = suite_ids(CHECK_SUITES);
        assert!(ids.contains(&"fmt"));
        assert!(ids.contains(&"lint"));
        assert!(ids.contains(&"dep-guard"));
    }

    #[test]
    fn test_suites_include_runtime_and_governance() {
        let ids = suite_ids(TEST_SUITES);
        assert!(ids.contains(&"unit"));
        assert!(ids.contains(&"arch"));
        assert!(ids.contains(&"evidence-consumer-integrity"));
    }

    #[test]
    fn contract_suites_include_adapter_and_backend_conformance() {
        let ids = suite_ids(CONTRACT_SUITES);
        assert!(ids.contains(&"adapter-conformance"));
        assert!(ids.contains(&"backend-conformance"));
    }

    #[test]
    fn release_suites_include_evidence_bundle_and_readiness() {
        let ids = suite_ids(RELEASE_SUITES);
        assert!(ids.contains(&"readiness"));
        assert!(ids.contains(&"evidence-bundle"));
    }
}
