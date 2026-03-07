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
