use std::path::PathBuf;
use std::process::Command;

fn examples_file(file_name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(file_name);
    root.to_string_lossy().into_owned()
}

#[test]
fn representative_json_error_snapshot_is_stable() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "lint",
            "--strict",
            "--json",
            &examples_file("hello.dag.json"),
        ])
        .output()
        .expect("run lint strict json");

    assert!(!output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json response");

    let snapshot = serde_json::json!({
        "ok": payload["ok"],
        "command": payload["command"],
        "error": {
            "category": payload["error"]["category"],
            "code": payload["error"]["code"],
            "exit_code": payload["error"]["exit_code"]
        }
    });

    let rendered = serde_json::to_string_pretty(&snapshot).expect("snapshot json");
    let expected = include_str!("snapshots/error_json_shape.json");
    assert_eq!(rendered.trim(), expected.trim());
}
