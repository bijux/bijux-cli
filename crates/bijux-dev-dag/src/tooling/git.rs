use std::process::Command;

pub fn git_status_porcelain() -> Result<String, String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err("git status --short failed".to_string());
    }
    String::from_utf8(output.stdout).map_err(|err| err.to_string())
}
