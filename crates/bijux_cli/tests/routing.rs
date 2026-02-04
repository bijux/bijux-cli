use std::process::Command;

#[test]
fn dag_validate_routes() {
    let bin = env!("CARGO_BIN_EXE_bijux");
    let dag = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("hello.dag.json");
    let out = Command::new(bin)
        .args(["dag", "validate", dag.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn unknown_subapp_fails() {
    let bin = env!("CARGO_BIN_EXE_bijux");
    let out = Command::new(bin).args(["foo"]).output().unwrap();
    assert!(!out.status.success());
}
