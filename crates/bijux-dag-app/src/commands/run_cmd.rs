use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct RunCommandResponse {
    pub run_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::RunCommandResponse;
    use std::path::PathBuf;

    #[test]
    fn run_command_response_serializes_run_dir() {
        let response = RunCommandResponse {
            run_dir: PathBuf::from("runs/run-1"),
        };
        let value = serde_json::to_value(&response).expect("serialize run response");
        assert_eq!(value["run_dir"], "runs/run-1");
    }
}
