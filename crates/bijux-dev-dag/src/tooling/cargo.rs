use std::process::Command;

pub fn cargo_status(args: &[&str]) -> Result<(), String> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo command failed: cargo {}", args.join(" ")))
    }
}
