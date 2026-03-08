use crate::report::model::CommandReport;
use std::fs;
use std::path::Path;

pub fn write_command_report(path: &Path, report: &CommandReport) -> Result<(), String> {
    let payload = serde_json::to_vec_pretty(report).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
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
}
