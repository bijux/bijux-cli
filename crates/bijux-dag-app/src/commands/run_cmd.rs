use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct RunCommandResponse {
    pub run_dir: PathBuf,
}
