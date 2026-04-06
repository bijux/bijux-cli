use crate::RuntimeError;

pub fn append_indexed_event(
    run_log: &mut std::fs::File,
    index: &mut Vec<serde_json::Value>,
    event: serde_json::Value,
) -> Result<(), RuntimeError> {
    crate::append_event(run_log, event.clone())?;
    index.push(event);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::append_indexed_event;
    use serde_json::json;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn appends_to_log_and_keeps_index_in_sync() {
        let file = NamedTempFile::new().expect("tmp file");
        let path = file.path().to_path_buf();
        let mut log = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .expect("open log");
        let mut index = Vec::new();
        let event = json!({"event":"node_started","node_id":"n1"});

        append_indexed_event(&mut log, &mut index, event.clone()).expect("append");
        assert_eq!(index.len(), 1);
        assert_eq!(index[0], event);

        let mut raw = String::new();
        std::fs::File::open(&path)
            .expect("read log")
            .read_to_string(&mut raw)
            .expect("log content");
        assert!(raw.contains("\"event\":\"node_started\""));
        assert!(raw.contains("\"node_id\":\"n1\""));
    }
}
