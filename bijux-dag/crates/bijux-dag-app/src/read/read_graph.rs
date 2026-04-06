use bijux_dag_core::{parse_graph_strict, Graph, GraphError, SPEC_VERSION};

pub fn parse_graph(input: &str) -> Result<Graph, GraphError> {
    parse_graph_strict(input)
}

pub fn parse_graph_with_compat(input: &str) -> Result<Graph, GraphError> {
    match parse_graph_strict(input) {
        Ok(g) => Ok(g),
        Err(GraphError::InvalidSpec(_)) => {
            let mut value = serde_json::from_str::<serde_json::Value>(input)?;
            if let Some(spec) = value.get("spec").and_then(serde_json::Value::as_str) {
                if spec == "0.1" || spec == "v0.1" {
                    value["spec"] = serde_json::Value::String(SPEC_VERSION.to_string());
                    let rewritten = serde_json::to_string(&value).map_err(GraphError::from)?;
                    return parse_graph_strict(&rewritten);
                }
            }
            Err(GraphError::InvalidSpec(format!(
                "unsupported spec version: {}",
                value
                    .get("spec")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("<missing>")
            )))
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_graph, parse_graph_with_compat};
    use bijux_dag_core::SPEC_VERSION;

    #[test]
    fn parses_graph_in_strict_mode() {
        let input = r#"{"spec":"v1","nodes":[],"edges":[]}"#;
        let graph = parse_graph(input).expect("parse graph");
        assert_eq!(graph.spec, SPEC_VERSION);
    }

    #[test]
    fn accepts_legacy_spec_in_compat_mode() {
        let input = r#"{"spec":"0.1","nodes":[],"edges":[]}"#;
        let graph = parse_graph_with_compat(input).expect("parse graph with compat");
        assert_eq!(graph.spec, SPEC_VERSION);
    }

    #[test]
    fn rejects_unknown_spec_in_compat_mode() {
        let input = r#"{"spec":"v9","nodes":[],"edges":[]}"#;
        assert!(parse_graph_with_compat(input).is_err());
    }
}
