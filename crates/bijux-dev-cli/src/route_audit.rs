//! Maintainer route-audit report assembly.

use serde_json::{json, Value};

/// Builds the maintainer route-audit report envelope from routing query rows.
#[must_use]
pub fn build_report_from_query(
    routes: &[Vec<String>],
    aliases: &[(Vec<String>, Vec<String>)],
) -> Value {
    let routes: Vec<Value> = routes
        .iter()
        .cloned()
        .map(|segments| json!({"segments": segments, "owner": "bijux-cli", "source": "built-in"}))
        .collect();
    let aliases: Vec<Value> = aliases
        .iter()
        .cloned()
        .map(|(alias, canonical)| {
            json!({"alias": alias, "canonical": canonical, "source": "compatibility-alias"})
        })
        .collect();
    let summary = json!({
        "route_count": routes.len(),
        "alias_count": aliases.len(),
    });

    json!({
        "routes": routes,
        "aliases": aliases,
        "summary": summary,
    })
}

#[cfg(test)]
mod tests {
    use super::build_report_from_query;

    #[test]
    fn route_audit_report_shape_is_stable() {
        let routes = vec![vec![
            "dev".to_string(),
            "cli".to_string(),
            "status".to_string(),
        ]];
        let aliases = vec![(
            vec!["status".to_string()],
            vec!["cli".to_string(), "status".to_string()],
        )];
        let report = build_report_from_query(&routes, &aliases);
        assert!(report.get("routes").is_some());
        assert!(report.get("aliases").is_some());
        assert!(report.get("summary").is_some());
    }
}
