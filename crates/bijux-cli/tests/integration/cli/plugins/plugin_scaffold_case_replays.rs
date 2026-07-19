#![forbid(unsafe_code)]
//! Plugin scaffold case replay suite for retained minimized scaffold inputs.
//! test_type: plugin-scaffold-case-replay

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run(args: &[String], plugins_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(args)
        .env("BIJUXCLI_PLUGINS_DIR", plugins_dir)
        .output()
        .expect("execute")
}

#[test]
fn minimized_scaffold_cases_replay_with_deterministic_exit_codes() {
    let dir = Path::new("tests/fuzz/minimized_cases/plugin_scaffold_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("scaffold minimized corpus must exist")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "argv"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "scaffold minimized corpus must not be empty");

    let root = std::env::temp_dir()
        .join(format!("bijux-plugin-scaffold-case-replay-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir root");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("plugins");

    for path in files {
        let text = fs::read_to_string(&path).expect("read case");
        let mut args: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect();
        for arg in &mut args {
            if arg.contains("{ROOT}") {
                *arg = arg.replace("{ROOT}", root.to_string_lossy().as_ref());
            }
        }

        let a = run(&args, &plugins_dir);
        let b = run(&args, &plugins_dir);
        assert_eq!(a.status.code(), b.status.code(), "case={}", path.display());
    }
}
