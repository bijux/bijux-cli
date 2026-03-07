use std::path::PathBuf;
use std::process::Command;

fn examples_file(file_name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(file_name);
    root.to_string_lossy().into_owned()
}

#[test]
fn json_error_output_contains_structured_fields() {
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
    assert_eq!(payload["ok"], false);
    assert!(payload["error"].is_object());
    assert!(payload["error"]["category"].is_string());
    assert!(payload["error"]["code"].is_string());
    assert!(payload["error"]["exit_code"].is_number());
}
