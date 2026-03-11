//! Maintainer route inventory report assembly.

use serde_json::{json, Value};

use crate::ReportContext;

/// Builds the maintainer route inventory report envelope from routing query rows.
#[must_use]
pub fn build_report_from_query(
    routes: &[Vec<String>],
    aliases: &[(Vec<String>, Vec<String>)],
    _context: &ReportContext,
) -> Value {
    let routes: Vec<Value> = routes
        .iter()
        .map(|segments| json!({"segments": segments, "owner": "bijux-cli", "source": "built-in"}))
        .collect();
    let aliases: Vec<Value> = aliases
        .iter()
        .map(|(alias, canonical)| {
            json!({"alias": alias, "canonical": canonical, "source": "compatibility-alias"})
        })
        .collect();

    json!({ "routes": routes, "aliases": aliases })
}

#[cfg(test)]
mod tests {
    use super::build_report_from_query;
    use crate::ReportContext;

    #[test]
    fn routes_report_shape_is_stable() {
        let context =
            ReportContext { generated_at: String::new(), data_source: "routing".to_string() };
        let routes = vec![vec!["dev".to_string(), "cli".to_string(), "status".to_string()]];
        let aliases =
            vec![(vec!["status".to_string()], vec!["cli".to_string(), "status".to_string()])];

        let report = build_report_from_query(&routes, &aliases, &context);
        assert!(report.get("routes").is_some(), "routes field must exist");
        assert!(report.get("aliases").is_some(), "aliases field must exist");
    }
}
