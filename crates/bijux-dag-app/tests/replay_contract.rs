use std::path::Path;

#[test]
fn replay_fixture_family_exists() {
    let root = Path::new("tests/e2e/replay/fixtures");
    for rel in [
        "match_case.json",
        "mismatch_case.json",
        "corruption_case.json",
        "unsupported_version_case.json",
    ] {
        assert!(root.join(rel).exists(), "missing replay fixture: {}", rel);
    }
}

#[test]
fn replay_battle_scenario_declares_mandatory_proof() {
    let payload = std::fs::read_to_string("tests/e2e/replay/replay_semantic_comparison.json")
        .expect("read replay battle scenario");
    let value: serde_json::Value =
        serde_json::from_str(&payload).expect("parse replay battle scenario");
    let assertions = value["assertions"].as_array().expect("assertions array");
    assert!(assertions.iter().any(|v| v == "replay_mandatory_proof"));
}
