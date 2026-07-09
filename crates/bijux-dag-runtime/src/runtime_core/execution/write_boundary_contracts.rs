use serde::{Deserialize, Serialize};

/// Filesystem write authorization result for a requested path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteBoundaryDecisionV1 {
    pub requested_path: String,
    pub normalized_path: String,
    pub allowed: bool,
    pub reason: String,
}

/// Enforce write boundaries under allowed runtime roots.
pub fn enforce_write_boundary(
    requested_path: &str,
    normalized_path: &str,
    allowed_roots: &[String],
    symlink_escaped: bool,
) -> Result<WriteBoundaryDecisionV1, String> {
    if requested_path.trim().is_empty() {
        return Err("requested_path must not be empty".to_string());
    }
    if normalized_path.trim().is_empty() {
        return Err("normalized_path must not be empty".to_string());
    }
    if allowed_roots.is_empty() {
        return Err("allowed_roots must not be empty".to_string());
    }
    if normalized_path.contains("..") {
        return Ok(WriteBoundaryDecisionV1 {
            requested_path: requested_path.to_string(),
            normalized_path: normalized_path.to_string(),
            allowed: false,
            reason: "path traversal detected".to_string(),
        });
    }
    if symlink_escaped {
        return Ok(WriteBoundaryDecisionV1 {
            requested_path: requested_path.to_string(),
            normalized_path: normalized_path.to_string(),
            allowed: false,
            reason: "symlink escape detected".to_string(),
        });
    }
    let allowed = allowed_roots
        .iter()
        .any(|root| normalized_path == root || normalized_path.starts_with(&format!("{root}/")));
    Ok(WriteBoundaryDecisionV1 {
        requested_path: requested_path.to_string(),
        normalized_path: normalized_path.to_string(),
        allowed,
        reason: if allowed {
            "path is within an allowed root".to_string()
        } else {
            "path is outside all allowed roots".to_string()
        },
    })
}

/// Environment allowlist filtering report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvAllowlistReportV1 {
    pub allowed_variables: Vec<(String, String)>,
    pub dropped_variables: Vec<String>,
}

/// Filter runtime environment by declared allowlist.
pub fn enforce_environment_allowlist(
    environment: Vec<(String, String)>,
    allowlist: &[String],
) -> Result<EnvAllowlistReportV1, String> {
    if allowlist.is_empty() {
        return Err("allowlist must not be empty".to_string());
    }
    let mut allowed_variables = Vec::new();
    let mut dropped_variables = Vec::new();
    for (key, value) in environment {
        if allowlist.iter().any(|allowed| allowed == &key) {
            allowed_variables.push((key, value));
        } else {
            dropped_variables.push(key);
        }
    }
    allowed_variables.sort_by(|left, right| left.0.cmp(&right.0));
    dropped_variables.sort();
    Ok(EnvAllowlistReportV1 { allowed_variables, dropped_variables })
}

/// Redaction report for sensitive text surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionReportV1 {
    pub redacted_text: String,
    pub redaction_count: usize,
}

/// Redact known secret values from logs/errors/support text.
pub fn redact_sensitive_values(
    input: &str,
    sensitive_values: &[String],
) -> Result<RedactionReportV1, String> {
    if input.is_empty() {
        return Err("input must not be empty".to_string());
    }
    let mut redacted_text = input.to_string();
    let mut redaction_count = 0usize;
    for secret in sensitive_values {
        if secret.trim().is_empty() {
            continue;
        }
        if redacted_text.contains(secret) {
            redacted_text = redacted_text.replace(secret, "[REDACTED]");
            redaction_count += 1;
        }
    }
    Ok(RedactionReportV1 { redacted_text, redaction_count })
}

/// Network policy labels for execution nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPolicyLabelV1 {
    Forbidden,
    Allowed,
    Required,
}

/// Trust class for cache/replay decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkTrustDecisionV1 {
    pub label: NetworkPolicyLabelV1,
    pub cache_trust: String,
    pub replay_trust: String,
    pub reason: String,
}

/// Classify cache/replay trust based on network policy and observation.
pub fn classify_network_policy_trust(
    label: NetworkPolicyLabelV1,
    network_observed: bool,
) -> NetworkTrustDecisionV1 {
    match (label.clone(), network_observed) {
        (NetworkPolicyLabelV1::Forbidden, true) => NetworkTrustDecisionV1 {
            label,
            cache_trust: "unsafe".to_string(),
            replay_trust: "unsafe".to_string(),
            reason: "network was observed despite forbidden policy".to_string(),
        },
        (NetworkPolicyLabelV1::Forbidden, false) => NetworkTrustDecisionV1 {
            label,
            cache_trust: "exact".to_string(),
            replay_trust: "exact".to_string(),
            reason: "network usage correctly absent".to_string(),
        },
        (NetworkPolicyLabelV1::Allowed, _) => NetworkTrustDecisionV1 {
            label,
            cache_trust: "compatible".to_string(),
            replay_trust: "compatible".to_string(),
            reason: "network optional under allowed policy".to_string(),
        },
        (NetworkPolicyLabelV1::Required, false) => NetworkTrustDecisionV1 {
            label,
            cache_trust: "unsafe".to_string(),
            replay_trust: "unsafe".to_string(),
            reason: "required network interaction was missing".to_string(),
        },
        (NetworkPolicyLabelV1::Required, true) => NetworkTrustDecisionV1 {
            label,
            cache_trust: "advisory".to_string(),
            replay_trust: "advisory".to_string(),
            reason: "required network interaction makes reproducibility conditional".to_string(),
        },
    }
}

/// Bundle import safety report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleImportSafetyReportV1 {
    pub accepted: bool,
    pub rejection_reasons: Vec<String>,
}

/// Validate untrusted bundle import constraints.
pub fn validate_bundle_import_safety(
    has_path_traversal: bool,
    has_malicious_symlink: bool,
    file_too_large: bool,
    corrupt_json: bool,
    schema_confusion: bool,
) -> BundleImportSafetyReportV1 {
    let mut rejection_reasons = Vec::new();
    if has_path_traversal {
        rejection_reasons.push("path_traversal".to_string());
    }
    if has_malicious_symlink {
        rejection_reasons.push("malicious_symlink".to_string());
    }
    if file_too_large {
        rejection_reasons.push("oversized_file".to_string());
    }
    if corrupt_json {
        rejection_reasons.push("corrupt_json".to_string());
    }
    if schema_confusion {
        rejection_reasons.push("schema_confusion".to_string());
    }
    BundleImportSafetyReportV1 { accepted: rejection_reasons.is_empty(), rejection_reasons }
}

/// Plugin execution authorization result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginExecutionAuthorizationV1 {
    pub plugin_path: String,
    pub allowed: bool,
    pub trust_class: String,
    pub reason: String,
}

/// Authorize plugin execution root and descriptor identity.
pub fn authorize_plugin_execution(
    plugin_path: &str,
    allowed_roots: &[String],
    descriptor_hash_expected: &str,
    descriptor_hash_observed: &str,
) -> Result<PluginExecutionAuthorizationV1, String> {
    if plugin_path.trim().is_empty() {
        return Err("plugin_path must not be empty".to_string());
    }
    if allowed_roots.is_empty() {
        return Err("allowed_roots must not be empty".to_string());
    }
    if descriptor_hash_expected.trim().is_empty() || descriptor_hash_observed.trim().is_empty() {
        return Err("descriptor hashes must not be empty".to_string());
    }

    let under_allowed_root = allowed_roots
        .iter()
        .any(|root| plugin_path == root || plugin_path.starts_with(&format!("{root}/")));
    if !under_allowed_root {
        return Ok(PluginExecutionAuthorizationV1 {
            plugin_path: plugin_path.to_string(),
            allowed: false,
            trust_class: "refused".to_string(),
            reason: "plugin path is outside allowed executable roots".to_string(),
        });
    }
    if descriptor_hash_expected != descriptor_hash_observed {
        return Ok(PluginExecutionAuthorizationV1 {
            plugin_path: plugin_path.to_string(),
            allowed: false,
            trust_class: "degraded".to_string(),
            reason: "plugin descriptor identity mismatch".to_string(),
        });
    }
    Ok(PluginExecutionAuthorizationV1 {
        plugin_path: plugin_path.to_string(),
        allowed: true,
        trust_class: "exact".to_string(),
        reason: "plugin execution authorized".to_string(),
    })
}

/// Bounded shell output capture report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellOutputCaptureReportV1 {
    pub stdout_bytes_captured: usize,
    pub stderr_bytes_captured: usize,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub overflow_artifact_pointer: Option<String>,
}

/// Capture shell output under bounded byte limits.
pub fn capture_shell_output_bounded(
    stdout_bytes_produced: usize,
    stderr_bytes_produced: usize,
    max_capture_bytes: usize,
) -> Result<ShellOutputCaptureReportV1, String> {
    if max_capture_bytes == 0 {
        return Err("max_capture_bytes must be positive".to_string());
    }
    let stdout_bytes_captured = stdout_bytes_produced.min(max_capture_bytes);
    let stderr_bytes_captured = stderr_bytes_produced.min(max_capture_bytes);
    let stdout_truncated = stdout_bytes_produced > max_capture_bytes;
    let stderr_truncated = stderr_bytes_produced > max_capture_bytes;
    let overflow_artifact_pointer = if stdout_truncated || stderr_truncated {
        Some("artifacts/runtime/shell-output-overflow.log".to_string())
    } else {
        None
    };
    Ok(ShellOutputCaptureReportV1 {
        stdout_bytes_captured,
        stderr_bytes_captured,
        stdout_truncated,
        stderr_truncated,
        overflow_artifact_pointer,
    })
}

/// Risky override event recorded by runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverrideAuditEventV1 {
    pub override_type: String,
    pub actor: String,
    pub reason: String,
}

/// Validate override audit stream for risky runtime choices.
pub fn validate_override_audit_events(
    events: Vec<OverrideAuditEventV1>,
) -> Result<Vec<OverrideAuditEventV1>, String> {
    if events.is_empty() {
        return Err("override audit events must not be empty".to_string());
    }
    for event in &events {
        for (field_name, field_value) in [
            ("override_type", event.override_type.as_str()),
            ("actor", event.actor.as_str()),
            ("reason", event.reason.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("override audit field {field_name} must not be empty"));
            }
        }
    }
    Ok(events)
}

/// Adapter software identity captured for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterSoftwareIdentityV1 {
    pub adapter_id: String,
    pub executable_path: String,
    pub version: String,
    pub binary_hash: String,
}

/// Supply-chain inventory report for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyChainInventoryReportV1 {
    pub run_id: String,
    pub adapters: Vec<AdapterSoftwareIdentityV1>,
    pub plugin_descriptor_ids: Vec<String>,
    pub app_descriptor_ids: Vec<String>,
}

/// Build per-run supply-chain inventory.
pub fn build_supply_chain_inventory_report(
    run_id: &str,
    adapters: Vec<AdapterSoftwareIdentityV1>,
    plugin_descriptor_ids: Vec<String>,
    app_descriptor_ids: Vec<String>,
) -> Result<SupplyChainInventoryReportV1, String> {
    if run_id.trim().is_empty() {
        return Err("run_id must not be empty".to_string());
    }
    if adapters.is_empty() {
        return Err("adapters must not be empty".to_string());
    }
    if plugin_descriptor_ids.is_empty() {
        return Err("plugin_descriptor_ids must not be empty".to_string());
    }
    if app_descriptor_ids.is_empty() {
        return Err("app_descriptor_ids must not be empty".to_string());
    }
    for adapter in &adapters {
        for (field_name, field_value) in [
            ("adapter_id", adapter.adapter_id.as_str()),
            ("executable_path", adapter.executable_path.as_str()),
            ("version", adapter.version.as_str()),
            ("binary_hash", adapter.binary_hash.as_str()),
        ] {
            if field_value.trim().is_empty() {
                return Err(format!("adapter identity field {field_name} must not be empty"));
            }
        }
    }
    Ok(SupplyChainInventoryReportV1 {
        run_id: run_id.to_string(),
        adapters,
        plugin_descriptor_ids,
        app_descriptor_ids,
    })
}

/// Runtime risk surface for one node under low-trust profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRiskSurfaceV1 {
    pub node_id: String,
    pub uses_plugin: bool,
    pub uses_shell: bool,
    pub uses_network: bool,
    pub broad_path_access: bool,
    pub exposes_secrets: bool,
}

/// Low-trust profile admission result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowTrustAdmissionReportV1 {
    pub admitted: bool,
    pub refused_nodes: Vec<(String, String)>,
}

/// Enforce low-trust runtime profile before execution starts.
pub fn enforce_low_trust_profile(
    nodes: Vec<NodeRiskSurfaceV1>,
) -> Result<LowTrustAdmissionReportV1, String> {
    if nodes.is_empty() {
        return Err("nodes must not be empty".to_string());
    }
    let mut refused_nodes = Vec::new();
    for node in nodes {
        if node.node_id.trim().is_empty() {
            return Err("node_id must not be empty".to_string());
        }
        if node.uses_plugin {
            refused_nodes
                .push((node.node_id.clone(), "plugins disabled in low-trust profile".to_string()));
        }
        if node.uses_shell {
            refused_nodes.push((
                node.node_id.clone(),
                "shell execution disabled in low-trust profile".to_string(),
            ));
        }
        if node.uses_network {
            refused_nodes.push((
                node.node_id.clone(),
                "network access disabled in low-trust profile".to_string(),
            ));
        }
        if node.broad_path_access {
            refused_nodes.push((
                node.node_id.clone(),
                "broad path access disabled in low-trust profile".to_string(),
            ));
        }
        if node.exposes_secrets {
            refused_nodes.push((
                node.node_id.clone(),
                "secret exposure disabled in low-trust profile".to_string(),
            ));
        }
    }
    Ok(LowTrustAdmissionReportV1 { admitted: refused_nodes.is_empty(), refused_nodes })
}

#[cfg(test)]
mod tests {
    use super::{
        authorize_plugin_execution, build_supply_chain_inventory_report,
        capture_shell_output_bounded, classify_network_policy_trust, enforce_environment_allowlist,
        enforce_low_trust_profile, enforce_write_boundary, redact_sensitive_values,
        validate_bundle_import_safety, validate_override_audit_events, AdapterSoftwareIdentityV1,
        NetworkPolicyLabelV1, NodeRiskSurfaceV1, OverrideAuditEventV1,
    };

    #[test]
    fn write_boundary_refuses_traversal_and_symlink_escape() {
        let allowed_roots = vec!["/workspace/runs".to_string(), "/workspace/cache".to_string()];
        let traversal = enforce_write_boundary(
            "../etc/passwd",
            "/workspace/runs/../etc/passwd",
            &allowed_roots,
            false,
        )
        .expect("traversal decision");
        assert!(!traversal.allowed);
        assert_eq!(traversal.reason, "path traversal detected");

        let symlink_escape = enforce_write_boundary(
            "runs/current/output.txt",
            "/workspace/runs/current/output.txt",
            &allowed_roots,
            true,
        )
        .expect("symlink decision");
        assert!(!symlink_escape.allowed);
        assert_eq!(symlink_escape.reason, "symlink escape detected");
    }

    #[test]
    fn environment_allowlist_prevents_secret_leakage() {
        let report = enforce_environment_allowlist(
            vec![
                ("PATH".to_string(), "/usr/bin".to_string()),
                ("WORKFLOW_ID".to_string(), "run-100".to_string()),
                ("API_TOKEN".to_string(), "secret-value".to_string()),
                ("SSH_PRIVATE_KEY".to_string(), "sensitive".to_string()),
            ],
            &["PATH".to_string(), "WORKFLOW_ID".to_string()],
        )
        .expect("allowlist report");
        assert_eq!(report.allowed_variables.len(), 2);
        assert!(report.dropped_variables.contains(&"API_TOKEN".to_string()));
        assert!(report.dropped_variables.contains(&"SSH_PRIVATE_KEY".to_string()));
    }

    #[test]
    fn secret_redaction_removes_sensitive_values_while_keeping_context() {
        let report = redact_sensitive_values(
            "request failed token=abc123 path=/workspace/run secret=topsecret",
            &["abc123".to_string(), "topsecret".to_string()],
        )
        .expect("redaction report");
        assert_eq!(report.redaction_count, 2);
        assert!(!report.redacted_text.contains("abc123"));
        assert!(!report.redacted_text.contains("topsecret"));
        assert!(report.redacted_text.contains("path=/workspace/run"));
    }

    #[test]
    fn network_policy_labels_drive_cache_and_replay_trust() {
        let forbidden = classify_network_policy_trust(NetworkPolicyLabelV1::Forbidden, true);
        assert_eq!(forbidden.cache_trust, "unsafe");
        assert_eq!(forbidden.replay_trust, "unsafe");

        let required = classify_network_policy_trust(NetworkPolicyLabelV1::Required, true);
        assert_eq!(required.cache_trust, "advisory");
        assert_eq!(required.replay_trust, "advisory");
    }

    #[test]
    fn bundle_import_rejects_hostile_inputs() {
        let report = validate_bundle_import_safety(true, true, false, true, true);
        assert!(!report.accepted);
        assert!(report.rejection_reasons.contains(&"path_traversal".to_string()));
        assert!(report.rejection_reasons.contains(&"malicious_symlink".to_string()));
        assert!(report.rejection_reasons.contains(&"corrupt_json".to_string()));
        assert!(report.rejection_reasons.contains(&"schema_confusion".to_string()));
    }

    #[test]
    fn plugin_execution_refuses_unknown_or_mutated_binaries() {
        let outside = authorize_plugin_execution(
            "/tmp/rogue/plugin",
            &["/workspace/plugins".to_string()],
            "hash-a",
            "hash-a",
        )
        .expect("outside root decision");
        assert!(!outside.allowed);
        assert_eq!(outside.trust_class, "refused");

        let mutated = authorize_plugin_execution(
            "/workspace/plugins/official/plugin",
            &["/workspace/plugins".to_string()],
            "hash-a",
            "hash-b",
        )
        .expect("mutated decision");
        assert!(!mutated.allowed);
        assert_eq!(mutated.trust_class, "degraded");
    }

    #[test]
    fn shell_output_capture_is_bounded_and_points_to_overflow_artifact() {
        let report = capture_shell_output_bounded(8192, 256, 1024).expect("capture report");
        assert_eq!(report.stdout_bytes_captured, 1024);
        assert_eq!(report.stderr_bytes_captured, 256);
        assert!(report.stdout_truncated);
        assert!(!report.stderr_truncated);
        assert!(report
            .overflow_artifact_pointer
            .as_deref()
            .unwrap_or_default()
            .contains("shell-output-overflow.log"));
    }

    #[test]
    fn override_audit_records_risky_runtime_choices() {
        let events = validate_override_audit_events(vec![
            OverrideAuditEventV1 {
                override_type: "forced_rerun".to_string(),
                actor: "operator@lab".to_string(),
                reason: "invalidated stale upstream artifact manually".to_string(),
            },
            OverrideAuditEventV1 {
                override_type: "cache_bypass".to_string(),
                actor: "operator@lab".to_string(),
                reason: "cache trust downgraded after backend incident".to_string(),
            },
            OverrideAuditEventV1 {
                override_type: "disabled_redaction".to_string(),
                actor: "security@lab".to_string(),
                reason: "temporary forensic exception with case ticket".to_string(),
            },
            OverrideAuditEventV1 {
                override_type: "unsafe_plugin_enablement".to_string(),
                actor: "operator@lab".to_string(),
                reason: "plugin not yet signed but needed for emergency pipeline".to_string(),
            },
            OverrideAuditEventV1 {
                override_type: "policy_weakening".to_string(),
                actor: "admin@lab".to_string(),
                reason: "temporary local-profile downgrade for incident triage".to_string(),
            },
        ])
        .expect("override audit");
        assert_eq!(events.len(), 5);
        assert!(events.iter().any(|event| event.override_type == "cache_bypass"));
    }

    #[test]
    fn supply_chain_inventory_captures_adapter_plugin_and_app_surfaces() {
        let report = build_supply_chain_inventory_report(
            "run-20260501-001",
            vec![
                AdapterSoftwareIdentityV1 {
                    adapter_id: "shell".to_string(),
                    executable_path: "/bin/sh".to_string(),
                    version: "5.2".to_string(),
                    binary_hash: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .to_string(),
                },
                AdapterSoftwareIdentityV1 {
                    adapter_id: "const".to_string(),
                    executable_path: "/workspace/bin/const-adapter".to_string(),
                    version: "0.4.0".to_string(),
                    binary_hash: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                        .to_string(),
                },
            ],
            vec!["plugin:official:quality-gate@1.2.0".to_string()],
            vec!["app:dag@0.4.0".to_string()],
        )
        .expect("supply chain inventory");
        assert_eq!(report.adapters.len(), 2);
        assert_eq!(report.plugin_descriptor_ids.len(), 1);
        assert_eq!(report.app_descriptor_ids.len(), 1);
    }

    #[test]
    fn low_trust_profile_refuses_unsafe_nodes_before_execution() {
        let report = enforce_low_trust_profile(vec![
            NodeRiskSurfaceV1 {
                node_id: "const-safe".to_string(),
                uses_plugin: false,
                uses_shell: false,
                uses_network: false,
                broad_path_access: false,
                exposes_secrets: false,
            },
            NodeRiskSurfaceV1 {
                node_id: "shell-risky".to_string(),
                uses_plugin: false,
                uses_shell: true,
                uses_network: true,
                broad_path_access: false,
                exposes_secrets: true,
            },
        ])
        .expect("low trust report");
        assert!(!report.admitted);
        assert!(report.refused_nodes.iter().any(|(node_id, _)| node_id == "shell-risky"));
    }
}
