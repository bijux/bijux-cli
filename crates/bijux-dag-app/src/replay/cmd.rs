use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ReplayCommandResponse {
    pub run_dir: PathBuf,
}
