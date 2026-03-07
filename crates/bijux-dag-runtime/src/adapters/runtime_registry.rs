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
        Self {
            by_kind: HashMap::new(),
        }
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
            list.push(AdapterInfo {
                adapter_id: id.id,
                adapter_version: id.version,
                effects,
            });
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
