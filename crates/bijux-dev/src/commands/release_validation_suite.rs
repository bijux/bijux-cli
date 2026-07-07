use crate::commands::model::{CommandContext, SuiteDef};
use crate::commands::reporting::run_text_or_json_report;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const SUITE_CONTRACT_PATH: &str = "configs/dag/release/release_validation_suite.json";
const DEFAULT_OPERATOR_HANDBOOK: &str = "docs/bijux-dev/operations/release-validation-suite.md";
const DEFAULT_WORKFLOW_HANDBOOK: &str = "docs/bijux-dev/gh-workflows/release-validation.md";
const DEFAULT_RELEASE_OPERATIONS_DOC: &str = "docs/bijux-dev/operations/release-operations.md";
const DEFAULT_MAINTAINER_ENTRYPOINT: &str =
    "cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ReleaseValidationSuiteContract {
    pub(crate) format: String,
    pub(crate) local_entrypoint: String,
    pub(crate) ci_entrypoint: String,
    pub(crate) package_boundary_contract: String,
    pub(crate) release_tree: ReleaseTreeContract,
    pub(crate) public_dag_crates: Vec<String>,
    pub(crate) commands: Vec<String>,
    #[serde(default)]
    pub(crate) maintainer_entrypoint: String,
    #[serde(default)]
    pub(crate) documentation: ReleaseValidationDocumentation,
    #[serde(default)]
    pub(crate) verify_flow: Vec<String>,
    #[serde(default)]
    pub(crate) artifacts: Vec<ReleaseValidationArtifact>,
    #[serde(default)]
    pub(crate) failure_ownership: Vec<ReleaseFailureOwnership>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ReleaseTreeContract {
    pub(crate) script: String,
    pub(crate) candidate_ref: String,
    pub(crate) version_source: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ReleaseValidationDocumentation {
    #[serde(default)]
    pub(crate) operator_handbook: String,
    #[serde(default)]
    pub(crate) workflow_handbook: String,
    #[serde(default)]
    pub(crate) release_operations: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ReleaseValidationArtifact {
    pub(crate) path: String,
    pub(crate) purpose: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ReleaseFailureOwnership {
    pub(crate) failure_class: String,
    pub(crate) owner: String,
    pub(crate) action: String,
}

pub(crate) fn load_release_validation_suite() -> Result<ReleaseValidationSuiteContract, String> {
    let root = super::ops::repo_root()?;
    load_release_validation_suite_from_path(&root.join(SUITE_CONTRACT_PATH))
}

pub(crate) fn run_release_suite_explain(
    context: &CommandContext,
    suite_id: &str,
    suites: &[SuiteDef],
) -> Result<(), String> {
    let suite = suites
        .iter()
        .find(|suite| suite.id == suite_id)
        .ok_or_else(|| format!("suite '{suite_id}' is unknown"))?;
    let data = build_release_suite_explain_data(suite)?;

    run_text_or_json_report(
        context,
        "release",
        "release.explain",
        suite.effect.label(),
        data,
        || Ok(()),
        false,
    )
}

fn build_release_suite_explain_data(suite: &SuiteDef) -> Result<Value, String> {
    let base = json!({
        "id": suite.id,
        "group": "release",
        "description": suite.description,
        "domain": suite.domain,
        "slow": suite.slow,
        "internal": suite.internal,
        "effect": suite.effect.label(),
    });

    match suite.id {
        "verify" => {
            let suite_contract = load_release_validation_suite()?;
            Ok(build_verify_suite_data(suite, suite_contract))
        }
        "readiness" => Ok(merge_release_follow_up(
            base,
            json!({
                "artifacts": [{
                    "path": "artifacts/release/readiness_report.json",
                    "purpose": "release readiness evidence aggregation for the current repository state"
                }],
                "docs": {
                    "release_operations": DEFAULT_RELEASE_OPERATIONS_DOC,
                    "operator_handbook": DEFAULT_OPERATOR_HANDBOOK,
                }
            }),
        )),
        "compatibility-matrix" => Ok(merge_release_follow_up(
            base,
            json!({
                "artifacts": [{
                    "path": "artifacts/release/compatibility_matrix.json",
                    "purpose": "generated compatibility matrix for supported schema fixtures"
                }],
                "docs": {
                    "release_operations": DEFAULT_RELEASE_OPERATIONS_DOC,
                    "operator_handbook": DEFAULT_OPERATOR_HANDBOOK,
                }
            }),
        )),
        "evidence-bundle" => Ok(merge_release_follow_up(
            base,
            json!({
                "artifacts": [{
                    "path": "artifacts/release/evidence_bundle.json",
                    "purpose": "release evidence bundle assembled from readiness and compatibility artifacts"
                }],
                "docs": {
                    "release_operations": DEFAULT_RELEASE_OPERATIONS_DOC,
                }
            }),
        )),
        _ => Ok(base),
    }
}

fn merge_release_follow_up(base: Value, extra: Value) -> Value {
    let mut merged = base;
    if let (Some(target), Some(source)) = (merged.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    merged
}

fn build_verify_suite_data(
    suite: &SuiteDef,
    suite_contract: ReleaseValidationSuiteContract,
) -> Value {
    json!({
        "id": suite.id,
        "group": "release",
        "description": suite.description,
        "domain": suite.domain,
        "slow": suite.slow,
        "internal": suite.internal,
        "effect": suite.effect.label(),
        "flow": suite_contract.verify_flow,
        "command_surface": {
            "local_entrypoint": suite_contract.local_entrypoint,
            "ci_entrypoint": suite_contract.ci_entrypoint,
            "maintainer_entrypoint": suite_contract.maintainer_entrypoint,
        },
        "release_tree": suite_contract.release_tree,
        "package_boundary_contract": suite_contract.package_boundary_contract,
        "public_dag_crates": suite_contract.public_dag_crates,
        "commands": suite_contract.commands,
        "docs": suite_contract.documentation,
        "artifacts": suite_contract.artifacts,
        "failure_ownership": suite_contract.failure_ownership,
    })
}

fn load_release_validation_suite_from_path(
    path: &Path,
) -> Result<ReleaseValidationSuiteContract, String> {
    let raw =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let suite: ReleaseValidationSuiteContract = serde_json::from_str(&raw)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    Ok(suite.with_defaults())
}

impl ReleaseValidationSuiteContract {
    fn with_defaults(mut self) -> Self {
        if self.maintainer_entrypoint.is_empty() {
            self.maintainer_entrypoint = DEFAULT_MAINTAINER_ENTRYPOINT.to_string();
        }
        if self.documentation.operator_handbook.is_empty() {
            self.documentation.operator_handbook = DEFAULT_OPERATOR_HANDBOOK.to_string();
        }
        if self.documentation.workflow_handbook.is_empty() {
            self.documentation.workflow_handbook = DEFAULT_WORKFLOW_HANDBOOK.to_string();
        }
        if self.documentation.release_operations.is_empty() {
            self.documentation.release_operations = DEFAULT_RELEASE_OPERATIONS_DOC.to_string();
        }
        if self.verify_flow.is_empty() {
            self.verify_flow =
                crate::suites::release_verify_suite_ids().into_iter().map(str::to_string).collect();
        }
        if self.artifacts.is_empty() {
            self.artifacts = default_artifacts();
        }
        if self.failure_ownership.is_empty() {
            self.failure_ownership = default_failure_ownership();
        }
        self
    }
}

fn default_artifacts() -> Vec<ReleaseValidationArtifact> {
    vec![
        ReleaseValidationArtifact {
            path: "artifacts/rust/release-validation/<run-id>/workspace/".to_string(),
            purpose: "clean release tree prepared from committed HEAD".to_string(),
        },
        ReleaseValidationArtifact {
            path: "artifacts/rust/release-validation/<run-id>/target/".to_string(),
            purpose: "shared target directory reused across release validation commands"
                .to_string(),
        },
        ReleaseValidationArtifact {
            path: "artifacts/rust/release-validation/<run-id>/".to_string(),
            purpose: "per-command release validation logs and run outputs".to_string(),
        },
        ReleaseValidationArtifact {
            path: "artifacts/release/readiness_report.json".to_string(),
            purpose: "release readiness evidence report emitted after validation".to_string(),
        },
        ReleaseValidationArtifact {
            path: "artifacts/release/compatibility_matrix.json".to_string(),
            purpose: "compatibility matrix generated from supported schema fixtures".to_string(),
        },
    ]
}

fn default_failure_ownership() -> Vec<ReleaseFailureOwnership> {
    vec![
        ReleaseFailureOwnership {
            failure_class: "formatter, clippy, test, doc, package, and publish failures".to_string(),
            owner: "release candidate".to_string(),
            action: "fix the candidate commit or its governed release inputs before tagging".to_string(),
        },
        ReleaseFailureOwnership {
            failure_class: "clean release-tree export failures".to_string(),
            owner: ".github/scripts/prepare_release_tree.py".to_string(),
            action: "repair the release-tree export logic so the candidate commit can be validated in isolation"
                .to_string(),
        },
        ReleaseFailureOwnership {
            failure_class: "CI wrapper or workflow setup failures".to_string(),
            owner: ".github/workflows/release-validation.yml and makes/gh.mk".to_string(),
            action: "repair the workflow or wrapper target so CI executes the same suite as local maintainers"
                .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::model::CommandEffect;

    fn pass() -> Result<(), String> {
        if std::env::var("BIJUX_DEV_DAG_FORCE_TEST_FAIL").ok().as_deref() == Some("1") {
            return Err("forced failure".to_string());
        }
        Ok(())
    }

    fn verify_suite_def() -> SuiteDef {
        SuiteDef {
            id: "verify",
            description: "canonical release validation plus readiness evidence",
            domain: "release",
            slow: true,
            internal: false,
            effect: CommandEffect::ReadWrite,
            run: pass,
        }
    }

    #[test]
    fn suite_contract_defaults_fill_operator_surface_metadata() {
        let temp = tempfile::tempdir().expect("tmp");
        let path = temp.path().join("release_validation_suite.json");
        fs::write(
            &path,
            r#"{
  "format": "release-validation-suite/v1",
  "local_entrypoint": "make release-validate-rs",
  "ci_entrypoint": "make gh-release-validate",
  "package_boundary_contract": "contracts/foundation/workspace_package_boundary.v1.json",
  "release_tree": {
    "script": ".github/scripts/prepare_release_tree.py",
    "candidate_ref": "HEAD",
    "version_source": "workspace.package.version"
  },
  "public_dag_crates": ["bijux-dag-core"],
  "commands": ["cargo fmt --all -- --check"]
}"#,
        )
        .expect("write suite");

        let suite = load_release_validation_suite_from_path(&path).expect("load suite");
        assert_eq!(suite.maintainer_entrypoint, DEFAULT_MAINTAINER_ENTRYPOINT);
        assert_eq!(suite.documentation.operator_handbook, DEFAULT_OPERATOR_HANDBOOK);
        assert_eq!(suite.documentation.workflow_handbook, DEFAULT_WORKFLOW_HANDBOOK);
        assert_eq!(suite.documentation.release_operations, DEFAULT_RELEASE_OPERATIONS_DOC);
        assert_eq!(
            suite.verify_flow,
            crate::suites::release_verify_suite_ids()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
        assert!(!suite.artifacts.is_empty());
        assert!(!suite.failure_ownership.is_empty());
    }

    #[test]
    fn verify_explain_payload_includes_canonical_release_details() {
        let suite_contract = ReleaseValidationSuiteContract {
            format: "release-validation-suite/v1".to_string(),
            local_entrypoint: "make release-validate-rs".to_string(),
            ci_entrypoint: "make gh-release-validate".to_string(),
            package_boundary_contract: "contracts/foundation/workspace_package_boundary.v1.json"
                .to_string(),
            release_tree: ReleaseTreeContract {
                script: ".github/scripts/prepare_release_tree.py".to_string(),
                candidate_ref: "HEAD".to_string(),
                version_source: "workspace.package.version".to_string(),
            },
            public_dag_crates: vec!["bijux-dag-core".to_string()],
            commands: vec!["cargo fmt --all -- --check".to_string()],
            maintainer_entrypoint: String::new(),
            documentation: ReleaseValidationDocumentation::default(),
            verify_flow: Vec::new(),
            artifacts: Vec::new(),
            failure_ownership: Vec::new(),
        }
        .with_defaults();

        let suite = verify_suite_def();
        let data = build_verify_suite_data(&suite, suite_contract);

        assert_eq!(data["command_surface"]["local_entrypoint"], "make release-validate-rs");
        assert_eq!(data["command_surface"]["ci_entrypoint"], "make gh-release-validate");
        assert_eq!(data["command_surface"]["maintainer_entrypoint"], DEFAULT_MAINTAINER_ENTRYPOINT);
        assert_eq!(
            data["docs"]["operator_handbook"],
            "docs/bijux-dev/operations/release-validation-suite.md"
        );
        assert_eq!(data["flow"][0], "release.validation-suite");
        assert_eq!(data["artifacts"][3]["path"], "artifacts/release/readiness_report.json");
        assert_eq!(data["failure_ownership"][0]["owner"], "release candidate");
    }
}
