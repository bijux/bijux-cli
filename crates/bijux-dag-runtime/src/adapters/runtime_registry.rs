use crate::adapter::Adapter;
use crate::external_adapter;
use crate::{AdapterInfo, RuntimeError};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AdapterRegistry {
    by_kind: HashMap<String, Arc<dyn Adapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self { by_kind: HashMap::new() }
    }

    pub fn register_adapter(&mut self, adapter: Arc<dyn Adapter>) -> Result<(), RuntimeError> {
        for kind in adapter.supported_kinds() {
            self.register(kind, Arc::clone(&adapter))?;
        }
        Ok(())
    }

    pub fn register(
        &mut self,
        kind: String,
        adapter: Arc<dyn Adapter>,
    ) -> Result<(), RuntimeError> {
        if kind.trim().is_empty() {
            return Err(RuntimeError::Executor("adapter kind must not be empty".to_string()));
        }
        if self.by_kind.contains_key(&kind) {
            return Err(RuntimeError::Executor(format!(
                "multiple adapters registered for kind {}",
                kind
            )));
        }
        self.by_kind.insert(kind, adapter);
        Ok(())
    }

    pub fn resolve(&self, kind: &str) -> Result<Arc<dyn Adapter>, RuntimeError> {
        self.by_kind
            .get(kind)
            .map(Arc::clone)
            .ok_or_else(|| RuntimeError::Executor("missing adapter".to_string()))
    }

    pub fn list(&self) -> Vec<AdapterInfo> {
        let mut list = Vec::new();
        for adapter in self.by_kind.values() {
            let id = adapter.id();
            let req = adapter.required_effects();
            let mut effects = Vec::new();
            if req.filesystem {
                effects.push("filesystem".to_string());
            }
            if req.env {
                effects.push("env".to_string());
            }
            if req.network {
                effects.push("network".to_string());
            }
            if req.clock {
                effects.push("clock".to_string());
            }
            list.push(AdapterInfo { adapter_id: id.id, adapter_version: id.version, effects });
        }
        list.sort_by(|a, b| a.adapter_id.cmp(&b.adapter_id));
        list
    }
}

pub fn build_registry(builtins: Vec<Arc<dyn Adapter>>) -> Result<AdapterRegistry, RuntimeError> {
    let mut registry = AdapterRegistry::new();
    for adapter in builtins {
        registry.register_adapter(adapter)?;
    }
    if let Ok(external) = external_adapter::discover_external_adapters() {
        for adapter in external {
            registry.register_adapter(adapter)?;
        }
    }
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AdapterId, EffectSet, NodeCtx, NodeResult};
    use std::sync::Arc;

    struct DummyAdapter {
        id: &'static str,
        version: &'static str,
        kinds: Vec<String>,
    }

    impl Adapter for DummyAdapter {
        fn id(&self) -> AdapterId {
            AdapterId { id: self.id.to_string(), version: self.version.to_string() }
        }

        fn supported_kinds(&self) -> Vec<String> {
            self.kinds.clone()
        }

        fn required_effects(&self) -> EffectSet {
            EffectSet::default()
        }

        fn produces_outputs_schema_version(&self) -> String {
            "v0.1".to_string()
        }

        fn execute(&self, _ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
            Err(RuntimeError::Executor("not used".to_string()))
        }
    }

    #[test]
    fn duplicate_kind_registration_is_rejected() {
        let mut registry = AdapterRegistry::new();
        registry
            .register_adapter(Arc::new(DummyAdapter {
                id: "a",
                version: "0.1",
                kinds: vec!["const".to_string()],
            }))
            .expect("first adapter registers");
        let err = registry
            .register_adapter(Arc::new(DummyAdapter {
                id: "b",
                version: "0.1",
                kinds: vec!["const".to_string()],
            }))
            .expect_err("duplicate kind should fail");
        assert!(format!("{err}").contains("multiple adapters registered for kind const"));
    }

    #[test]
    fn empty_kind_registration_is_rejected() {
        let mut registry = AdapterRegistry::new();
        let err = registry
            .register_adapter(Arc::new(DummyAdapter {
                id: "a",
                version: "0.1",
                kinds: vec!["".to_string()],
            }))
            .expect_err("empty kind should fail");
        assert!(format!("{err}").contains("adapter kind must not be empty"));
    }

    #[test]
    fn list_order_is_deterministic_by_adapter_id() {
        let mut registry = AdapterRegistry::new();
        registry
            .register_adapter(Arc::new(DummyAdapter {
                id: "z-adapter",
                version: "0.1",
                kinds: vec!["kind-z".to_string()],
            }))
            .expect("register z adapter");
        registry
            .register_adapter(Arc::new(DummyAdapter {
                id: "a-adapter",
                version: "0.1",
                kinds: vec!["kind-a".to_string()],
            }))
            .expect("register a adapter");
        let listed = registry.list();
        let ids: Vec<_> = listed.into_iter().map(|row| row.adapter_id).collect();
        assert_eq!(ids, vec!["a-adapter".to_string(), "z-adapter".to_string()]);
    }
}
