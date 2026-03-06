use std::path::PathBuf;
use std::process::Command;

fn examples_file(file_name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(file_name);
    root.to_string_lossy().into_owned()
}

#[test]
fn app_text_validate_output_contract() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "validate",
            &examples_file("hello.dag.json"),
        ])
        .output()
        .expect("run validate");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("status:"));
}

#[test]
fn app_json_validate_output_contract() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "validate",
            "--json",
            &examples_file("hello.dag.json"),
        ])
        .output()
        .expect("run validate json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json response");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"].is_object());
}
