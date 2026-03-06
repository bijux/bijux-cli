use crate::{Graph, ValidationDiagnostic};

pub fn validate_graph(graph: &Graph) -> Vec<ValidationDiagnostic> {
    graph.validate_with_warnings()
}
