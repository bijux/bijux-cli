#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::install::atomic_write_text;

use super::error::ConfigError;
use super::serialization::{decode_quoted_value, render_env};
use super::validation::{normalize_key, validate_value};

pub(crate) trait ConfigRepository {
    fn load(&self, path: &Path) -> Result<BTreeMap<String, String>, ConfigError>;
    fn save(&self, path: &Path, values: &BTreeMap<String, String>) -> Result<(), ConfigError>;
    fn remove(&self, path: &Path) -> Result<bool, ConfigError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FileConfigRepository;

impl ConfigRepository for FileConfigRepository {
    fn load(&self, path: &Path) -> Result<BTreeMap<String, String>, ConfigError> {
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let text =
            fs::read_to_string(path).map_err(|err| ConfigError::persistence(err.to_string()))?;
        let mut out = BTreeMap::new();
        for (index, raw_line) in text.lines().enumerate() {
            let line_no = index + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((raw_key, raw_value)) = raw_line.split_once('=') else {
                return Err(ConfigError::parse(format!("Malformed line {line_no}: {raw_line}")));
            };
            let key = normalize_key(raw_key)?;
            let value = decode_quoted_value(raw_value.trim());
            validate_value(&value)?;
            out.insert(key, value);
        }
        Ok(out)
    }

    fn save(&self, path: &Path, values: &BTreeMap<String, String>) -> Result<(), ConfigError> {
        let rendered = render_env(values);
        atomic_write_text(path, &rendered)
            .map_err(|err| ConfigError::persistence(err.to_string()))?;
        Ok(())
    }

    fn remove(&self, path: &Path) -> Result<bool, ConfigError> {
        if !path.exists() {
            return Ok(false);
        }
        fs::remove_file(path).map_err(|err| ConfigError::persistence(err.to_string()))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{ConfigRepository, FileConfigRepository};

    fn make_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let path = std::env::temp_dir().join(format!("bijux-storage-{name}-{nanos}"));
        fs::create_dir_all(&path).expect("mkdir");
        path
    }

    #[test]
    fn parser_handles_empty_and_missing_files() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("missing");
        let missing = temp.join("missing.env");
        let loaded = repo.load(&missing).expect("missing treated as empty");
        assert!(loaded.is_empty());

        let empty = temp.join("empty.env");
        fs::write(&empty, "").expect("write empty");
        let loaded_empty = repo.load(&empty).expect("empty parse");
        assert!(loaded_empty.is_empty());
    }

    #[test]
    fn parser_rejects_malformed_lines() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("malformed");
        let malformed = temp.join("malformed.env");
        fs::write(&malformed, "BIJUXCLI_OK=1\nMALFORMED\n").expect("write malformed");
        let err = repo.load(&malformed).expect_err("must fail");
        assert!(err.to_string().contains("Malformed line 2"));
    }

    #[test]
    fn parser_keeps_last_duplicate_key_value() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("dupes");
        let path = temp.join("dupes.env");
        fs::write(
            &path,
            "BIJUXCLI_ALPHA=1\nBIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=old\nBIJUXCLI_BETA=new\n",
        )
        .expect("write dupes");

        let loaded = repo.load(&path).expect("parse");
        assert_eq!(loaded.get("alpha").map(String::as_str), Some("1"));
        assert_eq!(loaded.get("beta").map(String::as_str), Some("new"));
    }

    #[test]
    fn parser_ignores_blank_and_comment_lines() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("comments");
        let path = temp.join("comments.env");
        fs::write(
            &path,
            "\n# top comment\nBIJUXCLI_ALPHA=1\n\n   # indented comment\nBIJUXCLI_BETA=2\n",
        )
        .expect("write comments");

        let loaded = repo.load(&path).expect("parse");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get("alpha").map(String::as_str), Some("1"));
        assert_eq!(loaded.get("beta").map(String::as_str), Some("2"));
    }

    #[test]
    fn parser_trims_trailing_whitespace_in_values() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("whitespace");
        let path = temp.join("whitespace.env");
        fs::write(&path, "BIJUXCLI_ALPHA=value   \n").expect("write");

        let loaded = repo.load(&path).expect("parse");
        assert_eq!(loaded.get("alpha").map(String::as_str), Some("value"));
    }

    #[test]
    fn writer_creates_parent_and_uses_deterministic_order() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("writer");
        let path = temp.join("nested").join("config.env");
        let mut map = BTreeMap::new();
        map.insert("beta".to_string(), "2".to_string());
        map.insert("alpha".to_string(), "1".to_string());

        repo.save(&path, &map).expect("save");
        let written = fs::read_to_string(&path).expect("read");
        assert_eq!(
            written, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n",
            "BTreeMap-backed rendering should stay stable"
        );
    }

    #[test]
    fn writer_drops_comments_and_formatting_by_design() {
        let repo = FileConfigRepository;
        let temp = make_temp_dir("rewrite");
        let path = temp.join("rewrite.env");
        fs::write(&path, "# comment\nBIJUXCLI_ALPHA=1\n").expect("seed");

        let loaded = repo.load(&path).expect("load");
        repo.save(&path, &loaded).expect("save");
        let rewritten = fs::read_to_string(&path).expect("read rewritten");
        assert_eq!(rewritten, "BIJUXCLI_ALPHA=1\n");
    }
}
