use bijux_dag_core::{parse_graph_strict, Graph};

pub fn parse_graph(input: &str) -> Result<Graph, bijux_dag_core::GraphError> {
    parse_graph_strict(input)
}
