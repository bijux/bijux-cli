use crate::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorField {
    Run,
    Graph,
    Node,
    NodePrefix,
    State,
    Tag,
    Artifact,
    Branch,
    Attempt,
    Kind,
    Id,
    IdPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorExpression {
    pub field: SelectorField,
    pub value: String,
}

pub(crate) fn parse_selector_expression(raw: &str) -> Result<SelectorExpression, ExitCode> {
    let (raw_key, raw_value) = raw.split_once(':').ok_or(ExitCode::from(2))?;
    let key = raw_key.trim().to_ascii_lowercase();
    let value = raw_value.trim();
    if value.is_empty() {
        return Err(ExitCode::from(2));
    }
    let field = match key.as_str() {
        "run" => SelectorField::Run,
        "graph" => SelectorField::Graph,
        "node" => SelectorField::Node,
        "node-prefix" => SelectorField::NodePrefix,
        "state" => SelectorField::State,
        "tag" => SelectorField::Tag,
        "artifact" => SelectorField::Artifact,
        "branch" => SelectorField::Branch,
        "attempt" => {
            if value.parse::<u32>().is_err() {
                return Err(ExitCode::from(2));
            }
            SelectorField::Attempt
        }
        "kind" => SelectorField::Kind,
        "id" => SelectorField::Id,
        "id-prefix" => SelectorField::IdPrefix,
        _ => return Err(ExitCode::from(2)),
    };
    Ok(SelectorExpression { field, value: value.to_string() })
}

pub(crate) fn parse_selector_expressions(
    values: &[String],
) -> Result<Vec<SelectorExpression>, ExitCode> {
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        out.push(parse_selector_expression(value)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_selector_expression, parse_selector_expressions, SelectorExpression, SelectorField,
    };
    use crate::ExitCode;

    #[test]
    fn grammar_parses_supported_selector_fields() {
        assert_eq!(
            parse_selector_expression("run:run-1").expect("run"),
            SelectorExpression { field: SelectorField::Run, value: "run-1".to_string() }
        );
        assert_eq!(
            parse_selector_expression("graph:workflow-a").expect("graph"),
            SelectorExpression { field: SelectorField::Graph, value: "workflow-a".to_string() }
        );
        assert_eq!(
            parse_selector_expression("node:align").expect("node"),
            SelectorExpression { field: SelectorField::Node, value: "align".to_string() }
        );
        assert_eq!(
            parse_selector_expression("node-prefix:train").expect("node-prefix"),
            SelectorExpression { field: SelectorField::NodePrefix, value: "train".to_string() }
        );
        assert_eq!(
            parse_selector_expression("state:failed").expect("state"),
            SelectorExpression { field: SelectorField::State, value: "failed".to_string() }
        );
        assert_eq!(
            parse_selector_expression("tag:etl").expect("tag"),
            SelectorExpression { field: SelectorField::Tag, value: "etl".to_string() }
        );
        assert_eq!(
            parse_selector_expression("artifact:report.json").expect("artifact"),
            SelectorExpression { field: SelectorField::Artifact, value: "report.json".to_string() }
        );
        assert_eq!(
            parse_selector_expression("branch:control").expect("branch"),
            SelectorExpression { field: SelectorField::Branch, value: "control".to_string() }
        );
        assert_eq!(
            parse_selector_expression("attempt:2").expect("attempt"),
            SelectorExpression { field: SelectorField::Attempt, value: "2".to_string() }
        );
        assert_eq!(
            parse_selector_expression("id-prefix:join").expect("id-prefix"),
            SelectorExpression { field: SelectorField::IdPrefix, value: "join".to_string() }
        );
    }

    #[test]
    fn grammar_rejects_invalid_selector_forms() {
        for selector in ["", "id", "id=", "unknown:value", "node:", "attempt:latest", "attempt:-1"]
        {
            let error =
                parse_selector_expression(selector).expect_err("invalid selector must fail");
            assert_eq!(error, ExitCode::from(2), "selector should reject: {selector}");
        }
    }

    #[test]
    fn grammar_parses_selector_lists_in_order() {
        let parsed = parse_selector_expressions(&[
            "run:r1".to_string(),
            "state:failed".to_string(),
            "tag:etl".to_string(),
        ])
        .expect("selectors");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].field, SelectorField::Run);
        assert_eq!(parsed[1].field, SelectorField::State);
        assert_eq!(parsed[2].field, SelectorField::Tag);
    }
}
