use crate::report::model::CommandReport;
use std::fs;
use std::path::Path;

pub fn write_command_report(path: &Path, report: &CommandReport) -> Result<(), String> {
    let payload = serde_json::to_vec_pretty(report).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
}

pub fn write_text_report(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, body).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_command_report_persists_json_payload() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("report.json");
        let report = CommandReport {
            command_id: "repo.check".to_string(),
            status: "ok".to_string(),
            effect: "validation".to_string(),
            started_unix_ms: 1,
            finished_unix_ms: 2,
            details: serde_json::json!({"ok": true}),
        };
        write_command_report(&path, &report).expect("write report");
        let loaded = fs::read_to_string(path).expect("read report");
        assert!(loaded.contains("\"command_id\": \"repo.check\""));
        assert!(loaded.contains("\"status\": \"ok\""));
    }

    #[test]
    fn write_text_report_persists_text_payload() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("report.md");
        write_text_report(&path, "# report\nok").expect("write text report");
        let loaded = fs::read_to_string(path).expect("read text report");
        assert!(loaded.contains("# report"));
    }

    #[test]
    fn write_command_report_is_idempotent_for_same_payload() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("report.json");
        let report = CommandReport {
            command_id: "repo.check".to_string(),
            status: "ok".to_string(),
            effect: "validation".to_string(),
            started_unix_ms: 1,
            finished_unix_ms: 2,
            details: serde_json::json!({"stable": true}),
        };
        write_command_report(&path, &report).expect("first write");
        let first = fs::read(&path).expect("first read");
        write_command_report(&path, &report).expect("second write");
        let second = fs::read(&path).expect("second read");
        assert_eq!(first, second, "report writing must be deterministic");
    }
}
