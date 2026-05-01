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
    Ok(EnvAllowlistReportV1 {
        allowed_variables,
        dropped_variables,
    })
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
    Ok(RedactionReportV1 {
        redacted_text,
        redaction_count,
    })
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
    BundleImportSafetyReportV1 {
        accepted: rejection_reasons.is_empty(),
        rejection_reasons,
    }
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

#[cfg(test)]
mod tests {
    use super::{
        capture_shell_output_bounded,
        authorize_plugin_execution,
        classify_network_policy_trust, enforce_environment_allowlist, enforce_write_boundary,
        redact_sensitive_values, validate_bundle_import_safety, NetworkPolicyLabelV1,
    };

    #[test]
    fn g081_write_boundary_refuses_traversal_and_symlink_escape() {
        let allowed_roots = vec![
            "/workspace/runs".to_string(),
            "/workspace/cache".to_string(),
        ];
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
    fn g082_environment_allowlist_prevents_secret_leakage() {
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
        assert!(report
            .dropped_variables
            .contains(&"API_TOKEN".to_string()));
        assert!(report
            .dropped_variables
            .contains(&"SSH_PRIVATE_KEY".to_string()));
    }

    #[test]
    fn g083_secret_redaction_removes_sensitive_values_while_keeping_context() {
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
    fn g084_network_policy_labels_drive_cache_and_replay_trust() {
        let forbidden = classify_network_policy_trust(NetworkPolicyLabelV1::Forbidden, true);
        assert_eq!(forbidden.cache_trust, "unsafe");
        assert_eq!(forbidden.replay_trust, "unsafe");

        let required = classify_network_policy_trust(NetworkPolicyLabelV1::Required, true);
        assert_eq!(required.cache_trust, "advisory");
        assert_eq!(required.replay_trust, "advisory");
    }

    #[test]
    fn g085_bundle_import_rejects_hostile_inputs() {
        let report = validate_bundle_import_safety(true, true, false, true, true);
        assert!(!report.accepted);
        assert!(report
            .rejection_reasons
            .contains(&"path_traversal".to_string()));
        assert!(report
            .rejection_reasons
            .contains(&"malicious_symlink".to_string()));
        assert!(report
            .rejection_reasons
            .contains(&"corrupt_json".to_string()));
        assert!(report
            .rejection_reasons
            .contains(&"schema_confusion".to_string()));
    }

    #[test]
    fn g086_plugin_execution_refuses_unknown_or_mutated_binaries() {
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
    fn g087_shell_output_capture_is_bounded_and_points_to_overflow_artifact() {
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
}
