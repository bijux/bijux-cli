use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub(crate) fn run_status(cmd: &str, args: &[&str]) -> Result<(), String> {
    run_status_in_dir(Path::new("."), cmd, args)
}

pub(crate) fn run_status_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {} {}", cmd, args.join(" ")))
    }
}

pub(crate) fn run_with_root(root: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    run_status_in_dir(root, cmd, args)
}

pub(crate) fn run_status_and_json(root: &Path, args: &[&str]) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "command failed: cargo {}",
            args.iter().copied().collect::<Vec<_>>().join(" ")
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

pub(crate) fn run_stdout_and_json(root: &Path, cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(format!("command failed: {} {}", cmd, args.join(" ")));
    }
    String::from_utf8(output.stdout).map_err(|err| err.to_string())
}

pub(crate) fn command_stdout(root: &Path, bin: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(bin)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "command failed: {} {}",
            bin,
            args.iter().copied().collect::<Vec<_>>().join(" ")
        ));
    }
    String::from_utf8(output.stdout).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{run_status, run_stdout_and_json};

    #[test]
    fn run_status_accepts_cargo_version_probe() {
        run_status("cargo", &["--version"]).expect("cargo --version");
    }

    #[test]
    fn run_stdout_returns_utf8() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let out = run_stdout_and_json(&root, "cargo", &["--version"]).expect("cargo --version");
        assert!(out.contains("cargo"));
    }
}
