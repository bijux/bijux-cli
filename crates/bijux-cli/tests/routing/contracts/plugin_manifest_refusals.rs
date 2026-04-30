#![forbid(unsafe_code)]

use std::path::PathBuf;

use bijux_cli::contracts::PluginManifestV2;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/fixtures/plugins").join(name)
}

#[test]
fn manifest_fixture_missing_trust_class_is_refused() {
    let text = std::fs::read_to_string(fixture_path("manifest_missing_trust_class_v2.json"))
        .expect("fixture");
    let error = serde_json::from_str::<PluginManifestV2>(&text)
        .expect_err("missing trust class must fail parsing");
    assert!(error.to_string().contains("trust_class"));
}

#[test]
fn manifest_fixture_invalid_trust_class_is_refused() {
    let text = std::fs::read_to_string(fixture_path("manifest_invalid_trust_class_v2.json"))
        .expect("fixture");
    let error = serde_json::from_str::<PluginManifestV2>(&text)
        .expect_err("invalid trust class must fail parsing");
    let message = error.to_string();
    assert!(message.contains("partner") || message.contains("trust"), "{message}");
}
