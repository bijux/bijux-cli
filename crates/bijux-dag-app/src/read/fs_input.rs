use std::fs;
use std::path::Path;

pub fn read_utf8_file(path: &Path) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

#[cfg(test)]
mod tests {
    use super::read_utf8_file;

    #[test]
    fn reads_utf8_file_from_filesystem() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let file = tmp.path().join("graph.json");
        std::fs::write(&file, "{\"spec\":\"v1\"}").expect("write file");
        let content = read_utf8_file(&file).expect("read file");
        assert_eq!(content, "{\"spec\":\"v1\"}");
    }

    #[test]
    fn read_utf8_file_errors_for_missing_file() {
        let missing = std::path::Path::new("/definitely/missing/bijux/dag.json");
        assert!(read_utf8_file(missing).is_err());
    }
}
