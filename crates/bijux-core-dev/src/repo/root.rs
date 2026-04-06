use std::path::PathBuf;

fn workspace_root_from(mut dir: PathBuf) -> Result<PathBuf, String> {
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("workspace root not found".to_string());
        }
    }
}

pub fn workspace_root() -> Result<PathBuf, String> {
    let dir = std::env::current_dir().map_err(|err| err.to_string())?;
    workspace_root_from(dir)
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

    #[test]
    fn workspace_root_is_stable_from_nested_workspace_paths() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = root.canonicalize().expect("canonical root");
        let nested = root.join("crates/bijux-core-dev/src");

        let from_root = workspace_root_from(root.clone()).expect("from root");
        let from_nested = workspace_root_from(nested).expect("from nested");
        assert_eq!(from_root, from_nested);
    }
}
