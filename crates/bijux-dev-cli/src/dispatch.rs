//! Maintainer command dispatch for `bijux dev cli ...`.

pub use crate::app::router::{owns_path, try_handle};
pub use crate::app::runtime_query::{
    ContractsSchemaInput, DoctorReportInput, RouteInventoryQuery, RuntimeQueryProvider,
    StateAuditInput,
};

#[cfg(test)]
mod tests {
    use super::owns_path;

    #[test]
    fn owns_path_matches_dev_cli_dispatch_surface() {
        assert!(owns_path(&["dev".into(), "cli".into(), "status".into()]));
        assert!(owns_path(&["dev".into(), "cli".into(), "maintenance".into(), "audit".into()]));
        assert!(owns_path(&["dev".into(), "cli".into(), "release".into(), "status".into()]));

        assert!(!owns_path(&["dev".into(), "status".into()]));
        assert!(!owns_path(&["cli".into(), "status".into()]));
        assert!(!owns_path(&["dev".into(), "cli".into(), "unknown".into(), "leaf".into()]));
    }
}
