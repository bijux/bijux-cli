use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
struct TrustPropertyTestMap {
    allowed_value_classes: Vec<String>,
    mappings: Vec<TrustMapping>,
}

#[derive(Debug, serde::Deserialize)]
struct TrustMapping {
    trust_property: String,
    value_class: String,
    tests: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn trust_properties_have_executable_test_mappings() {
    let root = repo_root();
    let map: TrustPropertyTestMap = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/trust_property_test_map.json"))
            .expect("read trust-property test map"),
    )
    .expect("parse trust-property test map");
    let battle_policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .expect("read battle trust policy"),
    )
    .expect("parse battle trust policy");

    let allowed: BTreeSet<String> = map.allowed_value_classes.into_iter().collect();
    let policy_trust: BTreeSet<String> = battle_policy["trust_properties"]
        .as_array()
        .expect("battle trust_properties")
        .iter()
        .map(|entry| entry["id"].as_str().expect("trust property id").to_string())
        .collect();

    let mut mapped = BTreeSet::new();
    for entry in map.mappings {
        assert!(
            policy_trust.contains(&entry.trust_property),
            "unknown trust property in mapping: {}",
            entry.trust_property
        );
        assert!(
            mapped.insert(entry.trust_property.clone()),
            "duplicate trust property mapping: {}",
            entry.trust_property
        );
        assert!(
            allowed.contains(&entry.value_class),
            "unknown test value class `{}` for trust property `{}`",
            entry.value_class,
            entry.trust_property
        );
        assert!(
            !entry.tests.is_empty(),
            "trust property must map to at least one test: {}",
            entry.trust_property
        );
        for rel in entry.tests {
            assert!(
                root.join(&rel).exists(),
                "mapped test file missing for trust property `{}`: {}",
                entry.trust_property,
                rel
            );
        }
    }

    for trust in policy_trust {
        assert!(
            mapped.contains(&trust),
            "missing trust-property-to-test mapping for `{trust}`"
        );
    }
}

#[test]
fn trust_property_test_report_is_present_and_mentions_focus_on_trust() {
    let root = repo_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/trust_property_to_test_report.md"))
            .expect("read trust-property-to-test report");
    assert!(
        report.contains("trust-proof coverage") || report.contains("trust property"),
        "report must center trust-property coverage language"
    );
    assert!(
        !report.contains("tests passed"),
        "report must not use raw pass-count bragging language"
    );
}
