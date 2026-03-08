pub fn node_output_relpath(node_id: &str, file: &str) -> String {
    crate::paths::node_output_relpath(node_id, file)
}

#[cfg(test)]
mod tests {
    use super::node_output_relpath;

    #[test]
    fn node_output_relpath_uses_nodes_outputs_layout() {
        assert_eq!(
            node_output_relpath("node-a", "result.bin"),
            "nodes/node-a/outputs/result.bin"
        );
    }
}
