use crate::{Graph, Severity, ValidationDiagnostic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationDomain {
    Schema,
    Semantic,
    Topology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationRule {
    pub id: &'static str,
    pub severity: Severity,
    pub domain: ValidationDomain,
}

pub fn validation_rule_registry() -> &'static [ValidationRule] {
    &[
        ValidationRule { id: "E1001", severity: Severity::Error, domain: ValidationDomain::Schema },
        ValidationRule { id: "E1002", severity: Severity::Error, domain: ValidationDomain::Topology },
        ValidationRule { id: "E1003", severity: Severity::Error, domain: ValidationDomain::Topology },
        ValidationRule { id: "E1004", severity: Severity::Error, domain: ValidationDomain::Topology },
        ValidationRule { id: "E1005", severity: Severity::Error, domain: ValidationDomain::Schema },
        ValidationRule { id: "E1006", severity: Severity::Error, domain: ValidationDomain::Schema },
        ValidationRule { id: "E1007", severity: Severity::Error, domain: ValidationDomain::Schema },
        ValidationRule { id: "E1008", severity: Severity::Error, domain: ValidationDomain::Topology },
        ValidationRule { id: "E1009", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1010", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1011", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1013", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1020", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1021", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1022", severity: Severity::Error, domain: ValidationDomain::Topology },
        ValidationRule { id: "E1023", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1024", severity: Severity::Error, domain: ValidationDomain::Semantic },
        ValidationRule { id: "E1025", severity: Severity::Error, domain: ValidationDomain::Schema },
        ValidationRule { id: "W2001", severity: Severity::Warning, domain: ValidationDomain::Topology },
        ValidationRule { id: "W2002", severity: Severity::Warning, domain: ValidationDomain::Topology },
    ]
}

pub fn validate_graph(graph: &Graph) -> Vec<ValidationDiagnostic> {
    graph.validate_with_warnings()
}

pub fn validate_schema(graph: &Graph) -> Vec<ValidationDiagnostic> {
    validate_graph(graph)
        .into_iter()
        .filter(|diag| {
            matches!(
                classify_rule_domain(diag.code.as_str()),
                Some(ValidationDomain::Schema)
            )
        })
        .collect()
}

pub fn validate_semantics(graph: &Graph) -> Vec<ValidationDiagnostic> {
    validate_graph(graph)
        .into_iter()
        .filter(|diag| {
            matches!(
                classify_rule_domain(diag.code.as_str()),
                Some(ValidationDomain::Semantic)
            )
        })
        .collect()
}

pub fn validate_topology(graph: &Graph) -> Vec<ValidationDiagnostic> {
    validate_graph(graph)
        .into_iter()
        .filter(|diag| {
            matches!(
                classify_rule_domain(diag.code.as_str()),
                Some(ValidationDomain::Topology)
            )
        })
        .collect()
}

fn classify_rule_domain(code: &str) -> Option<ValidationDomain> {
    validation_rule_registry()
        .iter()
        .find(|rule| rule.id == code)
        .map(|rule| rule.domain)
}
