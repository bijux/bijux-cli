use bijux_dag_artifacts::hash::sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FakeAdapterScenario {
    Success,
    Failure,
    Timeout,
    MissingOutput,
    CorruptOutput,
    LargeOutput,
    Panic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FakeAdapterExecution {
    pub scenario: FakeAdapterScenario,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub produced_outputs: BTreeMap<String, usize>,
    pub output_hashes: BTreeMap<String, String>,
    pub failure_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeAdapterHarness {
    output_name: String,
}

impl Default for FakeAdapterHarness {
    fn default() -> Self {
        Self::new("out.bin")
    }
}

impl FakeAdapterHarness {
    pub fn new(output_name: &str) -> Self {
        Self { output_name: output_name.to_string() }
    }

    pub fn execute(
        &self,
        scenario: FakeAdapterScenario,
        output_dir: &Path,
    ) -> Result<FakeAdapterExecution, String> {
        fs::create_dir_all(output_dir).map_err(|error| format!("create output dir: {error}"))?;
        match scenario {
            FakeAdapterScenario::Success => self.materialize(
                scenario,
                0,
                "fake adapter success\n",
                "",
                Some(b"ok\n".to_vec()),
                None,
                output_dir,
            ),
            FakeAdapterScenario::Failure => self.materialize(
                scenario,
                17,
                "",
                "fake adapter failure\n",
                None,
                Some("ExecutionFailed"),
                output_dir,
            ),
            FakeAdapterScenario::Timeout => self.materialize(
                scenario,
                124,
                "",
                "fake adapter timeout\n",
                None,
                Some("TimeoutExceeded"),
                output_dir,
            ),
            FakeAdapterScenario::MissingOutput => self.materialize(
                scenario,
                0,
                "missing declared output\n",
                "",
                None,
                Some("MissingRequiredOutput"),
                output_dir,
            ),
            FakeAdapterScenario::CorruptOutput => self.materialize(
                scenario,
                0,
                "corrupt output emitted\n",
                "",
                Some(vec![0xff, 0xfe, 0xfd, 0x00, b'x']),
                Some("ArtifactCorrupt"),
                output_dir,
            ),
            FakeAdapterScenario::LargeOutput => {
                let mut bytes = Vec::with_capacity(256 * 1024);
                bytes.resize(256 * 1024, b'x');
                self.materialize(
                    scenario,
                    0,
                    "large output emitted\n",
                    "",
                    Some(bytes),
                    None,
                    output_dir,
                )
            }
            FakeAdapterScenario::Panic => Err("fake adapter panic scenario".to_string()),
        }
    }

    pub fn render_report(&self, execution: &FakeAdapterExecution) -> Value {
        json!({
            "scenario": format!("{:?}", execution.scenario),
            "exit_code": execution.exit_code,
            "stdout": execution.stdout,
            "stderr": execution.stderr,
            "produced_outputs": execution.produced_outputs,
            "output_hashes": execution.output_hashes,
            "failure_kind": execution.failure_kind,
        })
    }

    fn materialize(
        &self,
        scenario: FakeAdapterScenario,
        exit_code: i32,
        stdout: &str,
        stderr: &str,
        payload: Option<Vec<u8>>,
        failure_kind: Option<&str>,
        output_dir: &Path,
    ) -> Result<FakeAdapterExecution, String> {
        let mut produced_outputs = BTreeMap::new();
        let mut output_hashes = BTreeMap::new();
        if let Some(bytes) = payload {
            let path = output_dir.join(&self.output_name);
            fs::write(&path, &bytes).map_err(|error| format!("write output: {error}"))?;
            produced_outputs.insert(self.output_name.clone(), bytes.len());
            output_hashes.insert(self.output_name.clone(), sha256_hex(&bytes));
        }
        Ok(FakeAdapterExecution {
            scenario,
            exit_code: Some(exit_code),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            produced_outputs,
            output_hashes,
            failure_kind: failure_kind.map(str::to_string),
        })
    }
}
