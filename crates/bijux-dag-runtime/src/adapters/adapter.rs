use crate::{NodeResult, RunContext, RuntimeError};
use bijux_dag_core::{Effect, Graph, Node};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterOrigin {
    BuiltIn,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheCompatibilityMode {
    FingerprintExact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterId {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSet {
    pub filesystem: bool,
    pub env: bool,
    pub network: bool,
    pub clock: bool,
}

impl EffectSet {
    pub fn from_effects(effects: &[Effect]) -> Self {
        let mut set = EffectSet::default();
        for e in effects {
            match e {
                Effect::Filesystem => set.filesystem = true,
                Effect::Env => set.env = true,
                Effect::Network => set.network = true,
                Effect::Clock => set.clock = true,
            }
        }
        set
    }
}

pub struct NodeCtx<'a> {
    pub graph: &'a Graph,
    pub node: &'a Node,
    pub exec: &'a RunContext,
    pub params: &'a serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterDescriptor {
    pub id: String,
    pub version: String,
    pub supported_kinds: Vec<String>,
    pub required_effects: EffectSet,
    pub produces_outputs_schema_version: String,
    pub origin: AdapterOrigin,
    pub protocol_version: String,
    pub cache_compatibility: CacheCompatibilityMode,
    pub supports_timeout: bool,
    pub supports_cancel: bool,
    pub binary_hash: Option<String>,
}

pub trait Adapter: Send + Sync {
    fn id(&self) -> AdapterId;
    fn supported_kinds(&self) -> Vec<String>;
    fn required_effects(&self) -> EffectSet;
    fn produces_outputs_schema_version(&self) -> String;
    fn protocol_version(&self) -> String {
        "bijux-dag-adapter/v1".to_string()
    }
    fn cache_compatibility(&self) -> CacheCompatibilityMode {
        CacheCompatibilityMode::FingerprintExact
    }
    fn supports_timeout(&self) -> bool {
        true
    }
    fn supports_cancel(&self) -> bool {
        false
    }
    fn origin(&self) -> AdapterOrigin {
        AdapterOrigin::BuiltIn
    }
    fn descriptor(&self) -> AdapterDescriptor {
        let id = self.id();
        AdapterDescriptor {
            id: id.id,
            version: id.version,
            supported_kinds: self.supported_kinds(),
            required_effects: self.required_effects(),
            produces_outputs_schema_version: self.produces_outputs_schema_version(),
            origin: self.origin(),
            protocol_version: self.protocol_version(),
            cache_compatibility: self.cache_compatibility(),
            supports_timeout: self.supports_timeout(),
            supports_cancel: self.supports_cancel(),
            binary_hash: self.binary_hash(),
        }
    }
    fn binary_hash(&self) -> Option<String> {
        None
    }
    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopAdapter;

    impl Adapter for NoopAdapter {
        fn id(&self) -> AdapterId {
            AdapterId { id: "noop".to_string(), version: "0.1".to_string() }
        }

        fn supported_kinds(&self) -> Vec<String> {
            vec!["const".to_string()]
        }

        fn required_effects(&self) -> EffectSet {
            EffectSet::default()
        }

        fn produces_outputs_schema_version(&self) -> String {
            "v0.1".to_string()
        }

        fn origin(&self) -> AdapterOrigin {
            AdapterOrigin::External
        }

        fn execute(&self, _ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
            Err(RuntimeError::Executor("not executed in this contract".to_string()))
        }
    }

    #[test]
    fn effect_set_maps_all_effects() {
        let set = EffectSet::from_effects(&[
            Effect::Filesystem,
            Effect::Env,
            Effect::Network,
            Effect::Clock,
        ]);
        assert!(set.filesystem);
        assert!(set.env);
        assert!(set.network);
        assert!(set.clock);
    }

    #[test]
    fn descriptor_contains_identity_origin_and_schema() {
        let adapter = NoopAdapter;
        let descriptor = adapter.descriptor();
        assert_eq!(descriptor.id, "noop");
        assert_eq!(descriptor.version, "0.1");
        assert_eq!(descriptor.supported_kinds, vec!["const".to_string()]);
        assert_eq!(descriptor.produces_outputs_schema_version, "v0.1");
        assert_eq!(descriptor.origin, AdapterOrigin::External);
        assert_eq!(descriptor.protocol_version, "bijux-dag-adapter/v1");
        assert_eq!(descriptor.cache_compatibility, CacheCompatibilityMode::FingerprintExact);
        assert!(descriptor.supports_timeout);
        assert!(!descriptor.supports_cancel);
    }
}
