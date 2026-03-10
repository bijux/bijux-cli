//! Maintainer registry report assembly.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::ReportContext;

/// Stable namespace row consumed by maintainer registry reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceInventoryRow {
    /// Namespace identifier.
    pub name: String,
    /// Reserved namespace marker.
    pub reserved: bool,
    /// Owning component.
    pub owner: String,
}

/// Builds the maintainer registry report envelope from routing query rows.
#[must_use]
pub fn build_report_from_query(
    namespaces: &[NamespaceInventoryRow],
    _context: &ReportContext,
) -> Value {
    let mut ownership: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in namespaces {
        ownership
            .entry(row.owner.clone())
            .or_default()
            .push(row.name.clone());
    }

    json!({
        "registry": namespaces.iter().map(|row| json!({
            "name": row.name,
            "reserved": row.reserved,
            "owner": row.owner,
        })).collect::<Vec<_>>(),
        "ownership": ownership,
        "precedence": ["reserved", "plugin"],
    })
}

#[cfg(test)]
mod tests {
    use super::{build_report_from_query, NamespaceInventoryRow};
    use crate::ReportContext;

    #[test]
    fn registry_report_shape_is_stable() {
        let context = ReportContext {
            generated_at: String::new(),
            data_source: "routing".to_string(),
        };
        let namespaces = vec![NamespaceInventoryRow {
            name: "dev".to_string(),
            reserved: true,
            owner: "bijux-cli".to_string(),
        }];

        let report = build_report_from_query(&namespaces, &context);
        assert!(
            report.get("registry").is_some(),
            "registry field must exist"
        );
        assert!(
            report.get("ownership").is_some(),
            "ownership field must exist"
        );
        assert!(
            report.get("precedence").is_some(),
            "precedence field must exist"
        );
    }
}
