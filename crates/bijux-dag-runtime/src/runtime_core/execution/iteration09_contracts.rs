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

#[cfg(test)]
mod tests {
    use super::enforce_write_boundary;

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
}
