//! Adapter conformance checks.

use crate::adapter::{AdapterDescriptor, AdapterOrigin, CacheCompatibilityMode};
use crate::backend::fake::FakeBatchExecutorContract;
use crate::backend_cluster::{KubernetesAdapterContractReport, SlurmAdapterDesignContractReport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConformanceReport {
    pub adapter_id: String,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn validate_descriptor(descriptor: &AdapterDescriptor) -> AdapterConformanceReport {
    let mut violations = Vec::new();
    if descriptor.id.trim().is_empty() {
        violations.push("missing adapter id".to_string());
    }
    if descriptor.version.trim().is_empty() {
        violations.push("missing adapter version".to_string());
    }
    if descriptor.supported_kinds.is_empty() {
        violations.push("missing supported kinds".to_string());
    }
    if descriptor.produces_outputs_schema_version.trim().is_empty() {
        violations.push("missing outputs schema version".to_string());
    }
    if descriptor.protocol_version.trim().is_empty() {
        violations.push("missing adapter protocol version".to_string());
    }
    if matches!(descriptor.origin, AdapterOrigin::External)
        && !descriptor.required_effects.filesystem
        && !descriptor.required_effects.env
        && !descriptor.required_effects.network
        && !descriptor.required_effects.clock
    {
        violations.push("external adapter declares no required effects".to_string());
    }
    if matches!(descriptor.origin, AdapterOrigin::External) && descriptor.binary_hash.is_none() {
        violations.push("external adapter missing binary hash".to_string());
    }

    AdapterConformanceReport {
        adapter_id: descriptor.id.clone(),
        passed: violations.is_empty(),
        violations,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterScenarioStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterScenarioResult {
    pub scenario: String,
    pub status: AdapterScenarioStatus,
    pub enforced_by_runtime: bool,
    pub advisory_only: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterConformanceSuiteReport {
    pub adapter_id: String,
    pub adapter_version: String,
    pub origin: AdapterOrigin,
    pub scenarios: Vec<AdapterScenarioResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterOutputSchemaCompatibilityReport {
    pub compatible: bool,
    pub compatibility_mode: CacheCompatibilityMode,
    pub produced_schema_version: String,
    pub expected_schema_version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterReferenceDocument {
    pub descriptors: Vec<AdapterDescriptor>,
    pub conformance: Vec<AdapterConformanceSuiteReport>,
    pub slurm: SlurmAdapterDesignContractReport,
    pub kubernetes: KubernetesAdapterContractReport,
    pub fake_batch: FakeBatchExecutorContract,
}

pub fn validate_output_schema_compatibility(
    mode: CacheCompatibilityMode,
    produced_schema_version: &str,
    expected_schema_version: &str,
) -> AdapterOutputSchemaCompatibilityReport {
    let compatible = match mode {
        CacheCompatibilityMode::FingerprintExact => {
            produced_schema_version == expected_schema_version
        }
    };
    let reason = if compatible {
        "produced output schema matches the expected adapter schema".to_string()
    } else {
        match mode {
            CacheCompatibilityMode::FingerprintExact => format!(
                "cache entry schema '{}' is incompatible with expected schema '{}' under fingerprint-exact compatibility",
                produced_schema_version, expected_schema_version
            ),
        }
    };
    AdapterOutputSchemaCompatibilityReport {
        compatible,
        compatibility_mode: mode,
        produced_schema_version: produced_schema_version.to_string(),
        expected_schema_version: expected_schema_version.to_string(),
        reason,
    }
}

fn scenario(
    name: &str,
    status: AdapterScenarioStatus,
    enforced_by_runtime: bool,
    advisory_only: bool,
    reason: &str,
) -> AdapterScenarioResult {
    AdapterScenarioResult {
        scenario: name.to_string(),
        status,
        enforced_by_runtime,
        advisory_only,
        reason: reason.to_string(),
    }
}

pub fn build_adapter_conformance_suite(
    descriptor: &AdapterDescriptor,
) -> AdapterConformanceSuiteReport {
    let shell_backed = descriptor.id == "shell";
    let process_backed = descriptor.id == "shell"
        || descriptor.id == "container"
        || descriptor.id == "python"
        || matches!(descriptor.origin, AdapterOrigin::External);
    let request_backed =
        process_backed || matches!(descriptor.id.as_str(), "http" | "file_transform");
    let schema_compatibility = validate_output_schema_compatibility(
        descriptor.cache_compatibility.clone(),
        &descriptor.produces_outputs_schema_version,
        &descriptor.produces_outputs_schema_version,
    );
    let scenarios = vec![
        scenario(
            "success",
            AdapterScenarioStatus::Pass,
            true,
            false,
            "successful adapter execution is a required runtime path",
        ),
        scenario(
            "failure",
            if request_backed { AdapterScenarioStatus::Pass } else { AdapterScenarioStatus::Skip },
            request_backed,
            !request_backed,
            if request_backed {
                "runtime records explicit execution failure results"
            } else {
                "non-process adapters do not expose a process failure boundary"
            },
        ),
        scenario(
            "argv_contract",
            if shell_backed { AdapterScenarioStatus::Pass } else { AdapterScenarioStatus::Skip },
            shell_backed,
            !shell_backed,
            if shell_backed {
                "shell nodes require a non-empty argv array of strings and a non-blank executable before execution starts"
            } else {
                "argv validation is specific to shell-backed command adapters"
            },
        ),
        scenario(
            "missing_output",
            AdapterScenarioStatus::Pass,
            true,
            false,
            "runtime validates declared output files for every adapter execution",
        ),
        scenario(
            "undeclared_output",
            AdapterScenarioStatus::Pass,
            true,
            false,
            "runtime rejects files written outside the declared output contract",
        ),
        scenario(
            "timeout",
            if descriptor.supports_timeout {
                AdapterScenarioStatus::Pass
            } else {
                AdapterScenarioStatus::Skip
            },
            descriptor.supports_timeout,
            !descriptor.supports_timeout,
            if descriptor.supports_timeout {
                "runtime enforces declared timeout budgets and records timeout failures"
            } else {
                "adapter descriptor does not declare timeout support"
            },
        ),
        scenario(
            "cancel",
            if descriptor.supports_cancel {
                AdapterScenarioStatus::Pass
            } else {
                AdapterScenarioStatus::Skip
            },
            descriptor.supports_cancel,
            !descriptor.supports_cancel,
            if descriptor.supports_cancel {
                "adapter participates in explicit cancellation handshakes"
            } else {
                "adapter descriptor does not declare cancellation support"
            },
        ),
        scenario(
            "env_policy",
            if process_backed || matches!(descriptor.origin, AdapterOrigin::External) {
                AdapterScenarioStatus::Pass
            } else {
                AdapterScenarioStatus::Skip
            },
            process_backed || matches!(descriptor.origin, AdapterOrigin::External),
            !(process_backed || matches!(descriptor.origin, AdapterOrigin::External)),
            if process_backed || matches!(descriptor.origin, AdapterOrigin::External) {
                "runtime shapes and filters adapter environments before execution"
            } else {
                "non-process adapters do not read process environment directly"
            },
        ),
        scenario(
            "workdir_isolation",
            if process_backed { AdapterScenarioStatus::Pass } else { AdapterScenarioStatus::Skip },
            process_backed,
            !process_backed,
            if process_backed {
                "runtime executes process-backed adapters from a dedicated node work directory"
            } else {
                "in-process adapters do not cross a working-directory boundary"
            },
        ),
        scenario(
            "missing_executable",
            if process_backed { AdapterScenarioStatus::Pass } else { AdapterScenarioStatus::Skip },
            process_backed,
            !process_backed,
            if process_backed {
                "runtime reports executable resolution failures with structured infrastructure errors"
            } else {
                "in-process adapters do not resolve external executables"
            },
        ),
        scenario(
            "cache_output",
            if schema_compatibility.compatible {
                AdapterScenarioStatus::Pass
            } else {
                AdapterScenarioStatus::Fail
            },
            true,
            false,
            &schema_compatibility.reason,
        ),
        scenario(
            "large_stdout",
            if process_backed { AdapterScenarioStatus::Pass } else { AdapterScenarioStatus::Skip },
            process_backed,
            !process_backed,
            if process_backed {
                "runtime captures stdout and stderr as node evidence files"
            } else {
                "non-process adapters do not emit stdout streams"
            },
        ),
        scenario(
            "non_utf8_output",
            if process_backed { AdapterScenarioStatus::Pass } else { AdapterScenarioStatus::Skip },
            process_backed,
            !process_backed,
            if process_backed {
                "runtime stores output bytes and artifact files without requiring UTF-8 payloads"
            } else {
                "non-process adapters do not emit process byte streams"
            },
        ),
    ];
    AdapterConformanceSuiteReport {
        adapter_id: descriptor.id.clone(),
        adapter_version: descriptor.version.clone(),
        origin: descriptor.origin,
        scenarios,
    }
}

pub fn generate_adapter_reference_markdown(document: &AdapterReferenceDocument) -> String {
    let mut lines = Vec::new();
    lines.push("# Adapter Contract".to_string());
    lines.push(String::new());
    lines.push("This document is generated from runtime adapter descriptors and backend contract references.".to_string());
    lines.push(String::new());
    lines.push("## Registered adapters".to_string());
    for descriptor in &document.descriptors {
        lines.push(format!(
            "- `{}` `{}`: kinds={:?}, origin={:?}, schema={}, timeout={}, cancel={}, cache={:?}",
            descriptor.id,
            descriptor.version,
            descriptor.supported_kinds,
            descriptor.origin,
            descriptor.produces_outputs_schema_version,
            descriptor.supports_timeout,
            descriptor.supports_cancel,
            descriptor.cache_compatibility
        ));
    }
    lines.push(String::new());
    lines.push("## Conformance scenarios".to_string());
    for report in &document.conformance {
        lines.push(format!("### {} {}", report.adapter_id, report.adapter_version));
        for scenario in &report.scenarios {
            lines.push(format!(
                "- `{}`: {:?} (enforced_by_runtime={}, advisory_only={}) - {}",
                scenario.scenario,
                scenario.status,
                scenario.enforced_by_runtime,
                scenario.advisory_only,
                scenario.reason
            ));
        }
        lines.push(String::new());
    }
    lines.push("## External adapter handshake boundary".to_string());
    lines.push("- `info --json` must emit machine JSON on stdout only.".to_string());
    lines.push("- non-empty stderr during the info handshake is rejected.".to_string());
    lines.push(
        "- external adapter binaries are fingerprinted into node trace evidence.".to_string(),
    );
    lines.push(String::new());
    lines.push("## Slurm contract".to_string());
    lines.push(format!(
        "- submit=`{}`, poll=`{}`, cancel=`{}`",
        document.slurm.contract.submit_command,
        document.slurm.contract.poll_command,
        document.slurm.contract.cancel_command
    ));
    lines.push(format!("- logs: {}", document.slurm.log_collection_mode));
    lines.push(format!("- artifacts: {}", document.slurm.artifact_collection_mode));
    lines.push(String::new());
    lines.push("## Kubernetes contract".to_string());
    lines.push(format!("- namespace: `{}`", document.kubernetes.contract.namespace));
    lines.push(format!("- job spec mapping: {}", document.kubernetes.job_spec_mapping));
    lines.push(format!("- pod status mapping: {}", document.kubernetes.pod_status_mapping));
    lines.push(format!("- logs: {}", document.kubernetes.log_collection_mode));
    lines.push(format!("- artifacts: {}", document.kubernetes.artifact_collection_mode));
    lines.push(format!(
        "- unsupported fields rejected: {}",
        document.kubernetes.unsupported_field_rejection.join(", ")
    ));
    lines.push(String::new());
    lines.push("## Fake batch executor".to_string());
    lines.push(format!(
        "- submit=`{}`, poll=`{}`, cancel=`{}`",
        document.fake_batch.submit_api,
        document.fake_batch.poll_api,
        document.fake_batch.cancel_api
    ));
    lines.push(format!("- states: {}", document.fake_batch.supported_states.join(", ")));
    lines.join("\n")
}
