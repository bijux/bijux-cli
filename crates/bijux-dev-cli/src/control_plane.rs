//! Maintainer control-plane helper report assembly.

use serde::Serialize;
use serde_json::{json, Value};

/// Builds hidden `dev cli atlas` report payload.
#[must_use]
pub fn build_atlas_report() -> Value {
    json!({"status": "ok", "mount": "atlas", "entry_surface": "dev-cli"})
}

/// Builds hidden `dev cli di` report payload.
#[must_use]
pub fn build_dependency_injection_report() -> Value {
    json!({"status": "ok", "container": "built-in", "entry_surface": "dev-cli"})
}

/// Builds hidden `dev cli list-products` report payload.
#[must_use]
pub fn build_product_list_report(products: &[&str]) -> Value {
    json!({"status": "ok", "products": products})
}

/// Builds hidden `dev cli list-plugins` report payload.
#[must_use]
pub fn build_plugin_list_report(plugins: Vec<Value>) -> Value {
    json!({"status": "ok", "plugins": plugins})
}

/// Builds hidden `dev cli list-plugins` report payload from structured plugin rows.
#[must_use]
pub fn build_plugin_list_report_from<T: Serialize>(plugins: T) -> Value {
    json!({"status": "ok", "plugins": plugins})
}

/// Builds `dev cli docs` report payload.
#[must_use]
pub fn build_docs_inventory_report(docs: Vec<String>) -> Value {
    json!({
        "docs_count": docs.len(),
        "docs": docs,
        "index": "docs/INDEX.md",
    })
}

/// Builds `dev cli docs-prune-plan` report payload.
#[must_use]
pub fn build_docs_prune_plan_report(docs_count: usize) -> Value {
    json!({
        "docs_count": docs_count,
        "target_cap": 60,
        "actions": [
            "merge overlapping architecture docs",
            "merge overlapping compatibility docs",
            "move low-value prose detail into generated JSON artifacts",
            "freeze docs rule: every doc explains law or change",
        ],
    })
}
