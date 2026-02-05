pub fn node_output_relpath(node_id: &str, file: &str) -> String {
    format!("nodes/{}/outputs/{}", node_id, file)
}
