use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

pub fn repo_root_from_manifest_dir(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir).join("../..").canonicalize().expect("workspace root")
}

pub fn run_dag_command(args: &[&str], cwd: &Path) -> (i32, String, String) {
    run_dag_command_with_env(args, cwd, &[])
}

pub fn run_dag_command_with_env(
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, &str)],
) -> (i32, String, String) {
    let output = Command::new(resolve_bijux_dag_binary(cwd))
        .current_dir(cwd)
        .envs(envs.iter().copied())
        .args(args)
        .output()
        .expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn resolve_bijux_dag_binary(cwd: &Path) -> PathBuf {
    static BIN_PATH: OnceLock<PathBuf> = OnceLock::new();
    BIN_PATH
        .get_or_init(|| {
            if let Some(path) = std::env::var_os("BIJUX_DAG_BIN").map(PathBuf::from) {
                if path.exists() {
                    return path;
                }
            }
            let workspace_root = resolve_workspace_root(cwd);
            let target_root = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace_root.join("artifacts").join("target"));
            let status = Command::new("cargo")
                .current_dir(&workspace_root)
                .env("RUSTFLAGS", "-Awarnings")
                .env("CARGO_TARGET_DIR", &target_root)
                .args(["build", "-q", "-p", "bijux-dag-cli"])
                .status()
                .expect("build bijux-dag binary");
            assert!(status.success(), "failed to build bijux-dag binary");
            target_root.join("debug").join(format!("bijux-dag{}", std::env::consts::EXE_SUFFIX))
        })
        .clone()
}

fn resolve_workspace_root(cwd: &Path) -> PathBuf {
    let mut current = cwd.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() && current.join("crates").exists() {
            return current;
        }
        if !current.pop() {
            panic!("unable to resolve workspace root from {}", cwd.display());
        }
    }
}
