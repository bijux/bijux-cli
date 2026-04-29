//! Adapter conformance checks.

use crate::adapter::{AdapterDescriptor, AdapterOrigin};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConformanceReport {
    pub adapter_id: String,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn validate_descriptor(descriptor: &AdapterDescriptor) -> AdapterConformanceReport {
    let mut violations = Vec::new();
    if descriptor.id.trim().is_empty() {
        violations.push("missing adapter id".to_string());
    }
    if descriptor.version.trim().is_empty() {
        violations.push("missing adapter version".to_string());
    }
    if descriptor.supported_kinds.is_empty() {
        violations.push("missing supported kinds".to_string());
    }
    if descriptor.produces_outputs_schema_version.trim().is_empty() {
        violations.push("missing outputs schema version".to_string());
    }
    if descriptor.protocol_version.trim().is_empty() {
        violations.push("missing adapter protocol version".to_string());
    }
    if matches!(descriptor.origin, AdapterOrigin::External)
        && !descriptor.required_effects.filesystem
        && !descriptor.required_effects.env
        && !descriptor.required_effects.network
        && !descriptor.required_effects.clock
    {
        violations.push("external adapter declares no required effects".to_string());
    }
    if matches!(descriptor.origin, AdapterOrigin::External) && descriptor.binary_hash.is_none() {
        violations.push("external adapter missing binary hash".to_string());
    }

    AdapterConformanceReport {
        adapter_id: descriptor.id.clone(),
        passed: violations.is_empty(),
        violations,
    }
}
