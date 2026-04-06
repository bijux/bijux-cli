use crate::adapter::AdapterDescriptor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCandidate {
    pub descriptor: AdapterDescriptor,
    pub compatibility_score: u32,
}

pub fn select_deterministic_adapter(candidates: &[AdapterCandidate]) -> Option<AdapterDescriptor> {
    candidates
        .iter()
        .max_by(|a, b| {
            a.compatibility_score
                .cmp(&b.compatibility_score)
                .then_with(|| b.descriptor.id.cmp(&a.descriptor.id))
                .then_with(|| b.descriptor.version.cmp(&a.descriptor.version))
        })
        .map(|candidate| candidate.descriptor.clone())
}

pub fn reject_duplicate_adapter_identity(descriptors: &[AdapterDescriptor]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for descriptor in descriptors {
        let identity = format!("{}@{}", descriptor.id, descriptor.version);
        if !seen.insert(identity.clone()) {
            return Err(format!("duplicate adapter identity: {identity}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        reject_duplicate_adapter_identity, select_deterministic_adapter, AdapterCandidate,
    };
    use crate::adapter::{AdapterDescriptor, AdapterOrigin, EffectSet};

    fn descriptor(id: &str, version: &str) -> AdapterDescriptor {
        AdapterDescriptor {
            id: id.to_string(),
            version: version.to_string(),
            supported_kinds: vec!["const".to_string()],
            required_effects: EffectSet::default(),
            produces_outputs_schema_version: "v0.1".to_string(),
            origin: AdapterOrigin::BuiltIn,
        }
    }

    #[test]
    fn deterministic_selection_prefers_score_then_identity() {
        let selected = select_deterministic_adapter(&[
            AdapterCandidate { descriptor: descriptor("b", "1.0"), compatibility_score: 10 },
            AdapterCandidate { descriptor: descriptor("a", "1.0"), compatibility_score: 10 },
            AdapterCandidate { descriptor: descriptor("z", "1.0"), compatibility_score: 9 },
        ])
        .expect("selected");

        assert_eq!(selected.id, "a");
        assert_eq!(selected.version, "1.0");
    }

    #[test]
    fn duplicate_identity_rejection_is_strict() {
        let err = reject_duplicate_adapter_identity(&[
            descriptor("shell", "1.0.0"),
            descriptor("shell", "1.0.0"),
        ])
        .expect_err("duplicate must fail");
        assert!(err.contains("duplicate adapter identity"));
    }
}
