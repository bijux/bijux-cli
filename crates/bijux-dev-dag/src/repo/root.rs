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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_finds_repo_root() {
        let root = workspace_root().expect("workspace root");
        assert!(root.join("Cargo.toml").exists());
        assert!(root.join("crates").is_dir());
    }
}
