use std::path::PathBuf;

pub fn workspace_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|err| err.to_string())?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("workspace root not found".to_string());
        }
    }
}
