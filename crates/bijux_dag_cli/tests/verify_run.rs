use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn verify_run_detects_corruption() {
    let bin = env!("CARGO_BIN_EXE_bijux-dag");
    let dir = tempdir().unwrap();
    let runs_dir = dir.path().join("runs");
    fs::create_dir_all(&runs_dir).unwrap();

    let dag = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("hello.dag.json");

    let out = Command::new(bin)
        .args([
            "run",
            dag.to_str().unwrap(),
            "--out",
            runs_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());

    let run_dir = fs::read_dir(&runs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("run-"))
        .unwrap();

    let out = Command::new(bin)
        .args(["verify-run", run_dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());

    let corrupt_path = run_dir
        .join("nodes")
        .join("const1")
        .join("outputs")
        .join("value.json");
    fs::write(&corrupt_path, b"corrupt").unwrap();

    let out = Command::new(bin)
        .args(["verify-run", run_dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
