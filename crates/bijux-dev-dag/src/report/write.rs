use crate::report::model::CommandReport;
use std::fs;
use std::path::Path;

pub fn write_command_report(path: &Path, report: &CommandReport) -> Result<(), String> {
    let payload = serde_json::to_vec_pretty(report).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
}
