use crate::adapter::{AdapterDescriptor, AdapterOrigin, EffectSet};
use crate::adapter_conformance::validate_descriptor;

#[test]
fn adapter_descriptor_requires_identity_and_schema_version() {
    let descriptor = AdapterDescriptor {
        id: "".to_string(),
        version: "".to_string(),
        supported_kinds: vec![],
        required_effects: EffectSet::default(),
        produces_outputs_schema_version: "".to_string(),
        origin: AdapterOrigin::BuiltIn,
    };
    let report = validate_descriptor(&descriptor);
    assert!(!report.passed);
    assert!(!report.violations.is_empty());
}

#[test]
fn external_adapter_requires_effect_declaration() {
    let descriptor = AdapterDescriptor {
        id: "ext".to_string(),
        version: "1".to_string(),
        supported_kinds: vec!["external.ext".to_string()],
        required_effects: EffectSet::default(),
        produces_outputs_schema_version: "v0.1".to_string(),
        origin: AdapterOrigin::External,
    };
    let report = validate_descriptor(&descriptor);
    assert!(!report.passed);
    assert!(report.violations.iter().any(|v| v.contains("declares no required effects")));
}

#[test]
fn built_in_descriptor_can_pass_with_minimum_contract() {
    let descriptor = AdapterDescriptor {
        id: "const".to_string(),
        version: "0.1".to_string(),
        supported_kinds: vec!["const".to_string()],
        required_effects: EffectSet::default(),
        produces_outputs_schema_version: "v0.1".to_string(),
        origin: AdapterOrigin::BuiltIn,
    };
    let report = validate_descriptor(&descriptor);
    assert!(report.passed);
    assert!(report.violations.is_empty());
}
