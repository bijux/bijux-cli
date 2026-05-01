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

#[cfg(test)]
mod tests {
    use super::{enforce_environment_allowlist, enforce_write_boundary, redact_sensitive_values};

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
}
