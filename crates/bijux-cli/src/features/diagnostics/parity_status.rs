#![forbid(unsafe_code)]
//! Parity/status artifact availability query model and resolver.

use std::path::Path;

/// Structured parity/status artifact availability queried from workspace state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityStatusQuery {
    /// Whether parity matrix artifact exists.
    pub command_parity_matrix_exists: bool,
    /// Whether runtime status artifact exists.
    pub status_report_exists: bool,
    /// Whether command migration matrix artifact exists.
    pub command_migration_matrix_exists: bool,
}

/// Query parity/status artifact availability from the workspace root.
#[must_use]
pub fn parity_status_query(workspace_root: &Path) -> ParityStatusQuery {
    ParityStatusQuery {
        command_parity_matrix_exists: workspace_root
            .join("artifacts/parity/command_parity_matrix.json")
            .exists(),
        status_report_exists: workspace_root.join("artifacts/status/status.json").exists(),
        command_migration_matrix_exists: workspace_root
            .join("artifacts/status/command_migration_matrix.json")
            .exists(),
    }
}
