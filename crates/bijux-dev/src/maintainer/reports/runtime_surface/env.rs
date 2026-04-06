//! Maintainer environment/source report assembly.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{json, Value};

/// Runtime-resolved active paths consumed by `bijux-dev-cli env`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePaths {
    /// Active config file path.
    pub config_file: PathBuf,
    /// Active history file path.
    pub history_file: PathBuf,
    /// Active plugins directory path.
    pub plugins_dir: PathBuf,
}

/// Builds the maintainer environment/source report envelope.
#[must_use]
pub fn build_report(env: BTreeMap<String, String>, active: &ActivePaths) -> Value {
    json!({
        "env": env,
        "source_precedence": ["flags", "env", "config", "defaults"],
        "active": {
            "config_file": active.config_file,
            "history_file": active.history_file,
            "plugins_dir": active.plugins_dir,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::{build_report, ActivePaths};

    #[test]
    fn env_report_shape_is_stable() {
        let mut env = BTreeMap::new();
        env.insert("BIJUX_CONFIG_PATH".to_string(), "/tmp/config".to_string());
        let active = ActivePaths {
            config_file: PathBuf::from("/tmp/config"),
            history_file: PathBuf::from("/tmp/history"),
            plugins_dir: PathBuf::from("/tmp/plugins"),
        };
        let report = build_report(env, &active);
        assert!(report.get("env").is_some());
        assert!(report.get("active").is_some());
        assert!(report.get("source_precedence").is_some());
    }
}
