pub fn node_output_relpath(node_id: &str, file: &str) -> String {
    format!("nodes/{}/outputs/{}", node_id, file)
}

pub fn is_normalized_relative_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if path.starts_with('/') {
        return false;
    }
    if path.contains('\\') {
        return false;
    }
    if path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return false;
    }
    true
}
