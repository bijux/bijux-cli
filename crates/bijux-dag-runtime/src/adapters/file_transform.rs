use crate::{
    authorize_input_path, Adapter, AdapterId, FailureClass, FailureInfo, NodeCtx, NodeResult,
    NodeStatus, RuntimeError,
};
use bijux_dag_artifacts::{sha256_artifact_path, write_outputs_index};
use flate2::read::GzDecoder;
use flate2::{Compression, GzBuilder};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

const OPERATION_SUMMARY_FILE: &str = "operation-summary.json";
const COPY_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Clone)]
pub struct FileTransformAdapter;

enum FileTransformOperation {
    Copy { source: String },
    Concatenate { sources: Vec<String> },
    Split { source: String, chunk_bytes: u64 },
    GzipCompress { source: String, compression_level: u32 },
    GzipDecompress { source: String },
    Checksum { source: String, algorithm: ChecksumAlgorithm },
}

enum ChecksumAlgorithm {
    Sha256,
}

#[derive(Debug, Serialize)]
struct FileTransformSummary {
    operation: String,
    sources: Vec<String>,
    outputs: Vec<FileTransformSummaryOutput>,
    bytes_read: u64,
    bytes_written: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    chunk_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    checksum: Option<ChecksumArtifact>,
}

#[derive(Debug, Serialize)]
struct FileTransformSummaryOutput {
    name: String,
    path: String,
    bytes: u64,
    sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_offset: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
struct ChecksumArtifact {
    operation: &'static str,
    algorithm: &'static str,
    source: String,
    bytes: u64,
    sha256: String,
}

struct OutputTarget {
    name: String,
    relative_path: String,
    absolute_path: PathBuf,
}

struct TransferProgress {
    bytes_copied: u64,
    source_exhausted: bool,
}

struct TimeoutBudget {
    started_at: Instant,
    timeout_ms: Option<u64>,
}

impl TimeoutBudget {
    fn new(timeout_ms: Option<u64>) -> Self {
        Self { started_at: Instant::now(), timeout_ms }
    }

    fn check(&self, operation: &str, phase: &str) -> Result<(), FailureInfo> {
        let Some(timeout_ms) = self.timeout_ms else {
            return Ok(());
        };
        if self.started_at.elapsed().as_millis() <= u128::from(timeout_ms) {
            return Ok(());
        }
        Err(FailureInfo::new(
            FailureClass::Timeout,
            "Timeout",
            "EXEC_TIMEOUT",
            "file_transform operation timed out after configured node timeout",
            Some(json!({
                "operation": operation,
                "phase": phase,
                "timeout_ms": timeout_ms,
            })),
        ))
    }
}

fn file_transform_user_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<Value>,
) -> FailureInfo {
    FailureInfo::new(FailureClass::User, "User", code.into(), message.into(), details)
}

fn file_transform_execution_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<Value>,
) -> FailureInfo {
    FailureInfo::new(FailureClass::Execution, "Execution", code.into(), message.into(), details)
}

fn file_transform_infrastructure_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<Value>,
) -> FailureInfo {
    FailureInfo::new(
        FailureClass::Infrastructure,
        "Infrastructure",
        code.into(),
        message.into(),
        details,
    )
}

fn failure_result(
    exec: &crate::RunContext,
    node_id: &str,
    status: NodeStatus,
    failure: FailureInfo,
    stdout_contents: &[u8],
    stderr_contents: &[u8],
) -> Result<NodeResult, RuntimeError> {
    let stdout_path = exec.run_dir.node_stdout_path(node_id);
    let stderr_path = exec.run_dir.node_stderr_path(node_id);
    let outputs_dir = exec.run_dir.node_outputs_dir(node_id);
    exec.fs.write(&stdout_path, stdout_contents)?;
    exec.fs.write(&stderr_path, stderr_contents)?;
    Ok(NodeResult {
        status,
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: outputs_dir.display().to_string(),
        output_evidence: Vec::new(),
        failure: Some(failure),
        attempts: 1,
        attempt_events: Vec::new(),
        container_meta: None,
        adapter_binary_sha256: None,
    })
}

fn parsed_operation(params: &Value) -> Result<FileTransformOperation, FailureInfo> {
    let Some(params) = params.as_object() else {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            "file_transform params must be an object",
            Some(json!({
                "field": "params",
                "reason": "expected_object",
            })),
        ));
    };

    let operation = match params.get("operation") {
        Some(Value::String(value)) if !value.trim().is_empty() => value.as_str(),
        Some(Value::String(_)) => {
            return Err(file_transform_user_failure(
                "EXEC_ERROR",
                "file_transform operation must not be empty",
                Some(json!({
                    "field": "operation",
                    "reason": "empty",
                })),
            ));
        }
        Some(_) => {
            return Err(file_transform_user_failure(
                "EXEC_ERROR",
                "file_transform operation must be a string",
                Some(json!({
                    "field": "operation",
                    "reason": "expected_string",
                })),
            ));
        }
        None => {
            return Err(file_transform_user_failure(
                "EXEC_ERROR",
                "file_transform operation is required",
                Some(json!({
                    "field": "operation",
                    "reason": "missing",
                })),
            ));
        }
    };

    match operation {
        "copy" => Ok(FileTransformOperation::Copy { source: required_string(params, "source")? }),
        "concatenate" => Ok(FileTransformOperation::Concatenate {
            sources: required_string_array(params, "sources")?,
        }),
        "split" => Ok(FileTransformOperation::Split {
            source: required_string(params, "source")?,
            chunk_bytes: required_positive_u64(params, "chunk_bytes")?,
        }),
        "gzip_compress" => Ok(FileTransformOperation::GzipCompress {
            source: required_string(params, "source")?,
            compression_level: optional_u32_range(params, "compression_level", 0, 9)?.unwrap_or(6),
        }),
        "gzip_decompress" => Ok(FileTransformOperation::GzipDecompress {
            source: required_string(params, "source")?,
        }),
        "checksum" => Ok(FileTransformOperation::Checksum {
            source: required_string(params, "source")?,
            algorithm: match params.get("checksum_algorithm") {
                None => ChecksumAlgorithm::Sha256,
                Some(Value::String(value)) if value == "sha256" => ChecksumAlgorithm::Sha256,
                Some(Value::String(_)) => {
                    return Err(file_transform_user_failure(
                        "EXEC_ERROR",
                        "file_transform checksum_algorithm must be sha256",
                        Some(json!({
                            "field": "checksum_algorithm",
                            "reason": "unsupported_algorithm",
                        })),
                    ));
                }
                Some(_) => {
                    return Err(file_transform_user_failure(
                        "EXEC_ERROR",
                        "file_transform checksum_algorithm must be a string",
                        Some(json!({
                            "field": "checksum_algorithm",
                            "reason": "expected_string",
                        })),
                    ));
                }
            },
        }),
        _ => Err(file_transform_user_failure(
            "EXEC_ERROR",
            "file_transform operation is unsupported",
            Some(json!({
                "field": "operation",
                "supported": [
                    "copy",
                    "concatenate",
                    "split",
                    "gzip_compress",
                    "gzip_decompress",
                    "checksum"
                ],
            })),
        )),
    }
}

fn required_string(
    params: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<String, FailureInfo> {
    match params.get(field) {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(value.clone()),
        Some(Value::String(_)) => Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must not be empty"),
            Some(json!({
                "field": field,
                "reason": "empty",
            })),
        )),
        Some(_) => Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be a string"),
            Some(json!({
                "field": field,
                "reason": "expected_string",
            })),
        )),
        None => Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} is required"),
            Some(json!({
                "field": field,
                "reason": "missing",
            })),
        )),
    }
}

fn required_string_array(
    params: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<Vec<String>, FailureInfo> {
    let Some(values) = params.get(field) else {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} is required"),
            Some(json!({
                "field": field,
                "reason": "missing",
            })),
        ));
    };
    let Some(values) = values.as_array() else {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be an array of strings"),
            Some(json!({
                "field": field,
                "reason": "expected_array",
            })),
        ));
    };
    if values.is_empty() {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must not be empty"),
            Some(json!({
                "field": field,
                "reason": "empty",
            })),
        ));
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            Value::String(text) if !text.trim().is_empty() => Ok(text.clone()),
            Value::String(_) => Err(file_transform_user_failure(
                "EXEC_ERROR",
                format!("file_transform {field}[{index}] must not be empty"),
                Some(json!({
                    "field": field,
                    "index": index,
                    "reason": "empty",
                })),
            )),
            _ => Err(file_transform_user_failure(
                "EXEC_ERROR",
                format!("file_transform {field}[{index}] must be a string"),
                Some(json!({
                    "field": field,
                    "index": index,
                    "reason": "expected_string",
                })),
            )),
        })
        .collect()
}

fn required_positive_u64(
    params: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<u64, FailureInfo> {
    match params.get(field).and_then(Value::as_u64) {
        Some(value) if value > 0 => Ok(value),
        Some(_) => Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be greater than zero"),
            Some(json!({
                "field": field,
                "reason": "non_positive",
            })),
        )),
        None if params.contains_key(field) => Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be an integer"),
            Some(json!({
                "field": field,
                "reason": "expected_integer",
            })),
        )),
        None => Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} is required"),
            Some(json!({
                "field": field,
                "reason": "missing",
            })),
        )),
    }
}

fn optional_u32_range(
    params: &serde_json::Map<String, Value>,
    field: &'static str,
    min: u32,
    max: u32,
) -> Result<Option<u32>, FailureInfo> {
    let Some(value) = params.get(field) else {
        return Ok(None);
    };
    let Some(value) = value.as_u64() else {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be an integer"),
            Some(json!({
                "field": field,
                "reason": "expected_integer",
            })),
        ));
    };
    let value = u32::try_from(value).map_err(|_| {
        file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be between {min} and {max}"),
            Some(json!({
                "field": field,
                "reason": "out_of_range",
                "minimum": min,
                "maximum": max,
            })),
        )
    })?;
    if !(min..=max).contains(&value) {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            format!("file_transform {field} must be between {min} and {max}"),
            Some(json!({
                "field": field,
                "reason": "out_of_range",
                "minimum": min,
                "maximum": max,
            })),
        ));
    }
    Ok(Some(value))
}

fn resolve_input_file(inputs_dir: &Path, relative_path: &str) -> Result<PathBuf, FailureInfo> {
    if !bijux_dag_artifacts::is_normalized_relative_path(relative_path) {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            "file_transform source path must be a normalized relative input path",
            Some(json!({
                "path": relative_path,
                "reason": "invalid_relative_path",
            })),
        ));
    }

    let candidate = inputs_dir.join(relative_path);
    let authorized = authorize_input_path(inputs_dir, &candidate).map_err(|error| {
        file_transform_user_failure(
            "EXEC_ERROR",
            "file_transform source path escapes the node inputs directory",
            Some(json!({
                "path": relative_path,
                "details": error,
            })),
        )
    })?;
    let metadata = fs::metadata(&authorized).map_err(|error| {
        let code = if error.kind() == io::ErrorKind::NotFound {
            "INPUT_MISSING"
        } else {
            "INPUT_READ_ERROR"
        };
        file_transform_infrastructure_failure(
            code,
            format!("file_transform source is unavailable: {relative_path}"),
            Some(json!({
                "path": relative_path,
                "details": error.to_string(),
            })),
        )
    })?;
    if !metadata.is_file() {
        return Err(file_transform_user_failure(
            "EXEC_ERROR",
            "file_transform source must resolve to a file",
            Some(json!({
                "path": relative_path,
            })),
        ));
    }
    Ok(authorized)
}

fn output_targets(
    node: &bijux_dag_core::Node,
    outputs_dir: &Path,
) -> Result<Vec<OutputTarget>, FailureInfo> {
    let mut targets = Vec::with_capacity(node.outputs.len());
    for output in &node.outputs {
        if output.expects_directory() {
            return Err(file_transform_user_failure(
                "OUTPUT_PATH_INVALID",
                format!(
                    "file_transform adapter cannot materialize directory output: {}",
                    output.path
                ),
                Some(json!({
                    "output": output.name,
                    "path": output.path,
                })),
            ));
        }
        let absolute_path = crate::authorized_declared_output_path(outputs_dir, output)?;
        targets.push(OutputTarget {
            name: output.name.clone(),
            relative_path: output.path.clone(),
            absolute_path,
        });
    }
    Ok(targets)
}

fn create_output_file(path: &Path) -> Result<File, FailureInfo> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            file_transform_infrastructure_failure(
                "OUTPUT_CREATE_ERROR",
                format!("failed to create output directory for {}", path.display()),
                Some(json!({
                    "path": path.display().to_string(),
                    "details": error.to_string(),
                })),
            )
        })?;
    }
    File::create(path).map_err(|error| {
        file_transform_infrastructure_failure(
            "OUTPUT_CREATE_ERROR",
            format!("failed to create output file {}", path.display()),
            Some(json!({
                "path": path.display().to_string(),
                "details": error.to_string(),
            })),
        )
    })
}

fn open_input_file(path: &Path) -> Result<File, FailureInfo> {
    File::open(path).map_err(|error| {
        file_transform_infrastructure_failure(
            "INPUT_READ_ERROR",
            format!("failed to open input file {}", path.display()),
            Some(json!({
                "path": path.display().to_string(),
                "details": error.to_string(),
            })),
        )
    })
}

fn io_failure(operation: &str, phase: &str, path: &Path, error: io::Error) -> FailureInfo {
    file_transform_execution_failure(
        "FILE_OPERATION_ERROR",
        format!("file_transform {operation} failed during {phase}"),
        Some(json!({
            "operation": operation,
            "phase": phase,
            "path": path.display().to_string(),
            "details": error.to_string(),
        })),
    )
}

fn copy_reader_to_writer<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    operation: &str,
    phase: &str,
    source_path: &Path,
    timeout: &TimeoutBudget,
    max_bytes: Option<u64>,
) -> Result<TransferProgress, FailureInfo> {
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    let mut copied = 0_u64;

    loop {
        timeout.check(operation, phase)?;
        let read_limit =
            max_bytes.map(|limit| limit.saturating_sub(copied)).unwrap_or(COPY_BUFFER_BYTES as u64);
        if read_limit == 0 {
            return Ok(TransferProgress { bytes_copied: copied, source_exhausted: false });
        }

        let chunk_len =
            usize::try_from(read_limit.min(COPY_BUFFER_BYTES as u64)).unwrap_or(COPY_BUFFER_BYTES);
        let read_count = reader
            .read(&mut buffer[..chunk_len])
            .map_err(|error| io_failure(operation, phase, source_path, error))?;
        if read_count == 0 {
            return Ok(TransferProgress { bytes_copied: copied, source_exhausted: true });
        }
        writer
            .write_all(&buffer[..read_count])
            .map_err(|error| io_failure(operation, phase, source_path, error))?;
        copied += u64::try_from(read_count).unwrap_or(0);
    }
}

fn output_summary(
    target: &OutputTarget,
    source_offset: Option<u64>,
) -> Result<FileTransformSummaryOutput, FailureInfo> {
    let metadata = fs::metadata(&target.absolute_path).map_err(|error| {
        file_transform_infrastructure_failure(
            "OUTPUT_STAT_ERROR",
            format!("failed to inspect output file {}", target.absolute_path.display()),
            Some(json!({
                "path": target.absolute_path.display().to_string(),
                "details": error.to_string(),
            })),
        )
    })?;
    let sha256 = sha256_artifact_path(&target.absolute_path).map_err(|error| {
        file_transform_infrastructure_failure(
            "OUTPUT_STAT_ERROR",
            format!("failed to hash output file {}", target.absolute_path.display()),
            Some(json!({
                "path": target.absolute_path.display().to_string(),
                "details": error.to_string(),
            })),
        )
    })?;
    Ok(FileTransformSummaryOutput {
        name: target.name.clone(),
        path: target.relative_path.clone(),
        bytes: metadata.len(),
        sha256,
        source_offset,
    })
}

fn checksum_source(
    source_path: &Path,
    timeout: &TimeoutBudget,
) -> Result<(u64, String), FailureInfo> {
    let mut reader = BufReader::new(open_input_file(source_path)?);
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    let mut bytes = 0_u64;

    loop {
        timeout.check("checksum", "read_source")?;
        let read_count = reader
            .read(&mut buffer)
            .map_err(|error| io_failure("checksum", "read_source", source_path, error))?;
        if read_count == 0 {
            break;
        }
        digest.update(&buffer[..read_count]);
        bytes += u64::try_from(read_count).unwrap_or(0);
    }

    Ok((bytes, hex::encode(digest.finalize())))
}

impl Adapter for FileTransformAdapter {
    fn id(&self) -> AdapterId {
        AdapterId { id: "file_transform".to_string(), version: "0.1".to_string() }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["file_transform".to_string()]
    }

    fn required_effects(&self) -> crate::EffectSet {
        crate::EffectSet { filesystem: true, env: false, network: false, clock: false }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&outputs_dir)?;
        if let Err(failure) = crate::preflight_declared_output_targets(&outputs_dir, &node.outputs)
        {
            let stderr = failure.message.clone();
            return failure_result(
                exec,
                &node.id,
                NodeStatus::Failed,
                failure,
                b"",
                stderr.as_bytes(),
            );
        }

        let operation = match parsed_operation(ctx.params) {
            Ok(operation) => operation,
            Err(failure) => {
                let stderr = failure.message.clone();
                return failure_result(
                    exec,
                    &node.id,
                    NodeStatus::Failed,
                    failure,
                    b"",
                    stderr.as_bytes(),
                );
            }
        };
        let targets = match output_targets(node, &outputs_dir) {
            Ok(targets) => targets,
            Err(failure) => {
                let stderr = failure.message.clone();
                return failure_result(
                    exec,
                    &node.id,
                    NodeStatus::Failed,
                    failure,
                    b"",
                    stderr.as_bytes(),
                );
            }
        };
        let timeout = TimeoutBudget::new(crate::effective_node_timeout_ms(node, ctx.params));
        let inputs_dir = exec.run_dir.node_inputs_dir(&node.id);

        let summary = (|| -> Result<FileTransformSummary, FailureInfo> {
            match operation {
                FileTransformOperation::Copy { source } => {
                    if targets.len() != 1 {
                        return Err(file_transform_user_failure(
                            "EXEC_ERROR",
                            "file_transform copy requires exactly one declared output",
                            Some(json!({
                                "declared_outputs": targets.len(),
                            })),
                        ));
                    }
                    let source_path = resolve_input_file(&inputs_dir, &source)?;
                    let output_path = &targets[0].absolute_path;
                    let mut reader = BufReader::new(open_input_file(&source_path)?);
                    let mut writer = BufWriter::new(create_output_file(output_path)?);
                    let progress = copy_reader_to_writer(
                        &mut reader,
                        &mut writer,
                        "copy",
                        "copy_file",
                        &source_path,
                        &timeout,
                        None,
                    )?;
                    writer
                        .flush()
                        .map_err(|error| io_failure("copy", "flush_output", output_path, error))?;
                    let output = output_summary(&targets[0], None)?;
                    Ok(FileTransformSummary {
                        operation: "copy".to_string(),
                        sources: vec![source],
                        outputs: vec![output],
                        bytes_read: progress.bytes_copied,
                        bytes_written: progress.bytes_copied,
                        chunk_bytes: None,
                        checksum: None,
                    })
                }
                FileTransformOperation::Concatenate { sources } => {
                    if targets.len() != 1 {
                        return Err(file_transform_user_failure(
                            "EXEC_ERROR",
                            "file_transform concatenate requires exactly one declared output",
                            Some(json!({
                                "declared_outputs": targets.len(),
                            })),
                        ));
                    }
                    let output_path = &targets[0].absolute_path;
                    let mut writer = BufWriter::new(create_output_file(output_path)?);
                    let mut total_read = 0_u64;
                    for source in &sources {
                        let source_path = resolve_input_file(&inputs_dir, source)?;
                        let mut reader = BufReader::new(open_input_file(&source_path)?);
                        let progress = copy_reader_to_writer(
                            &mut reader,
                            &mut writer,
                            "concatenate",
                            "copy_source",
                            &source_path,
                            &timeout,
                            None,
                        )?;
                        total_read += progress.bytes_copied;
                    }
                    writer.flush().map_err(|error| {
                        io_failure("concatenate", "flush_output", output_path, error)
                    })?;
                    let output = output_summary(&targets[0], None)?;
                    Ok(FileTransformSummary {
                        operation: "concatenate".to_string(),
                        sources,
                        bytes_read: total_read,
                        bytes_written: output.bytes,
                        outputs: vec![output],
                        chunk_bytes: None,
                        checksum: None,
                    })
                }
                FileTransformOperation::Split { source, chunk_bytes } => {
                    if targets.is_empty() {
                        return Err(file_transform_user_failure(
                            "EXEC_ERROR",
                            "file_transform split requires one or more declared outputs",
                            Some(json!({ "declared_outputs": 0 })),
                        ));
                    }
                    let source_path = resolve_input_file(&inputs_dir, &source)?;
                    let mut reader = BufReader::new(open_input_file(&source_path)?);
                    let mut outputs = Vec::new();
                    let mut bytes_read = 0_u64;
                    let mut offset = 0_u64;

                    for target in &targets {
                        timeout.check("split", "prepare_output")?;
                        let mut writer = BufWriter::new(create_output_file(&target.absolute_path)?);
                        let progress = copy_reader_to_writer(
                            &mut reader,
                            &mut writer,
                            "split",
                            "write_chunk",
                            &source_path,
                            &timeout,
                            Some(chunk_bytes),
                        )?;
                        writer.flush().map_err(|error| {
                            io_failure("split", "flush_chunk", &target.absolute_path, error)
                        })?;
                        if progress.bytes_copied == 0 {
                            fs::remove_file(&target.absolute_path).map_err(|error| {
                                file_transform_infrastructure_failure(
                                    "OUTPUT_CREATE_ERROR",
                                    format!(
                                        "failed to remove empty split output {}",
                                        target.absolute_path.display()
                                    ),
                                    Some(json!({
                                        "path": target.absolute_path.display().to_string(),
                                        "details": error.to_string(),
                                    })),
                                )
                            })?;
                            break;
                        }
                        outputs.push(output_summary(target, Some(offset))?);
                        offset += progress.bytes_copied;
                        bytes_read += progress.bytes_copied;
                        if progress.source_exhausted {
                            break;
                        }
                    }

                    let mut probe = [0_u8; 1];
                    timeout.check("split", "detect_trailing_bytes")?;
                    let trailing = reader.read(&mut probe).map_err(|error| {
                        io_failure("split", "detect_trailing_bytes", &source_path, error)
                    })?;
                    if trailing > 0 {
                        return Err(file_transform_execution_failure(
                            "SPLIT_OUTPUTS_EXHAUSTED",
                            "file_transform split declared too few outputs for the source file",
                            Some(json!({
                                "source": source,
                                "declared_outputs": targets.len(),
                                "chunk_bytes": chunk_bytes,
                            })),
                        ));
                    }

                    let bytes_written = outputs.iter().map(|output| output.bytes).sum();
                    Ok(FileTransformSummary {
                        operation: "split".to_string(),
                        sources: vec![source],
                        outputs,
                        bytes_read,
                        bytes_written,
                        chunk_bytes: Some(chunk_bytes),
                        checksum: None,
                    })
                }
                FileTransformOperation::GzipCompress { source, compression_level } => {
                    if targets.len() != 1 {
                        return Err(file_transform_user_failure(
                            "EXEC_ERROR",
                            "file_transform gzip_compress requires exactly one declared output",
                            Some(json!({
                                "declared_outputs": targets.len(),
                            })),
                        ));
                    }
                    let source_path = resolve_input_file(&inputs_dir, &source)?;
                    let output_path = &targets[0].absolute_path;
                    let mut reader = BufReader::new(open_input_file(&source_path)?);
                    let writer = BufWriter::new(create_output_file(output_path)?);
                    let mut encoder = GzBuilder::new()
                        .mtime(0)
                        .write(writer, Compression::new(compression_level));
                    let progress = copy_reader_to_writer(
                        &mut reader,
                        &mut encoder,
                        "gzip_compress",
                        "compress_stream",
                        &source_path,
                        &timeout,
                        None,
                    )?;
                    let mut writer = encoder.finish().map_err(|error| {
                        io_failure("gzip_compress", "finish_stream", output_path, error)
                    })?;
                    writer.flush().map_err(|error| {
                        io_failure("gzip_compress", "flush_output", output_path, error)
                    })?;
                    let output = output_summary(&targets[0], None)?;
                    let bytes_written = output.bytes;
                    Ok(FileTransformSummary {
                        operation: "gzip_compress".to_string(),
                        sources: vec![source],
                        outputs: vec![output],
                        bytes_read: progress.bytes_copied,
                        bytes_written,
                        chunk_bytes: None,
                        checksum: None,
                    })
                }
                FileTransformOperation::GzipDecompress { source } => {
                    if targets.len() != 1 {
                        return Err(file_transform_user_failure(
                            "EXEC_ERROR",
                            "file_transform gzip_decompress requires exactly one declared output",
                            Some(json!({
                                "declared_outputs": targets.len(),
                            })),
                        ));
                    }
                    let source_path = resolve_input_file(&inputs_dir, &source)?;
                    let bytes_read = fs::metadata(&source_path)
                        .map_err(|error| {
                            io_failure("gzip_decompress", "stat_source", &source_path, error)
                        })?
                        .len();
                    let reader = BufReader::new(open_input_file(&source_path)?);
                    let mut decoder = GzDecoder::new(reader);
                    let output_path = &targets[0].absolute_path;
                    let mut writer = BufWriter::new(create_output_file(output_path)?);
                    let _progress = copy_reader_to_writer(
                        &mut decoder,
                        &mut writer,
                        "gzip_decompress",
                        "decompress_stream",
                        &source_path,
                        &timeout,
                        None,
                    )?;
                    writer.flush().map_err(|error| {
                        io_failure("gzip_decompress", "flush_output", output_path, error)
                    })?;
                    let output = output_summary(&targets[0], None)?;
                    let bytes_written = output.bytes;
                    Ok(FileTransformSummary {
                        operation: "gzip_decompress".to_string(),
                        sources: vec![source],
                        outputs: vec![output],
                        bytes_read,
                        bytes_written,
                        chunk_bytes: None,
                        checksum: None,
                    })
                }
                FileTransformOperation::Checksum { source, algorithm } => {
                    if targets.len() != 1 {
                        return Err(file_transform_user_failure(
                            "EXEC_ERROR",
                            "file_transform checksum requires exactly one declared output",
                            Some(json!({
                                "declared_outputs": targets.len(),
                            })),
                        ));
                    }
                    let source_path = resolve_input_file(&inputs_dir, &source)?;
                    let (bytes, sha256) = match algorithm {
                        ChecksumAlgorithm::Sha256 => checksum_source(&source_path, &timeout)?,
                    };
                    let artifact = ChecksumArtifact {
                        operation: "checksum",
                        algorithm: "sha256",
                        source: source.clone(),
                        bytes,
                        sha256,
                    };
                    if let Some(parent) = targets[0].absolute_path.parent() {
                        fs::create_dir_all(parent).map_err(|error| {
                            file_transform_infrastructure_failure(
                                "OUTPUT_CREATE_ERROR",
                                format!(
                                    "failed to create checksum output directory {}",
                                    parent.display()
                                ),
                                Some(json!({
                                    "path": parent.display().to_string(),
                                    "details": error.to_string(),
                                })),
                            )
                        })?;
                    }
                    let artifact_bytes = serde_json::to_vec_pretty(&artifact).map_err(|error| {
                        file_transform_execution_failure(
                            "SERDE_ERROR",
                            "file_transform checksum artifact could not be serialized",
                            Some(json!({
                                "details": error.to_string(),
                            })),
                        )
                    })?;
                    fs::write(&targets[0].absolute_path, artifact_bytes).map_err(|error| {
                        file_transform_infrastructure_failure(
                            "OUTPUT_CREATE_ERROR",
                            format!(
                                "failed to write checksum output {}",
                                targets[0].absolute_path.display()
                            ),
                            Some(json!({
                                "path": targets[0].absolute_path.display().to_string(),
                                "details": error.to_string(),
                            })),
                        )
                    })?;
                    let output = output_summary(&targets[0], None)?;
                    let bytes_written = output.bytes;
                    Ok(FileTransformSummary {
                        operation: "checksum".to_string(),
                        sources: vec![source],
                        outputs: vec![output],
                        bytes_read: bytes,
                        bytes_written,
                        chunk_bytes: None,
                        checksum: Some(artifact),
                    })
                }
            }
        })();
        let summary = match summary {
            Ok(summary) => summary,
            Err(failure) => {
                let stderr = failure.message.clone();
                return failure_result(
                    exec,
                    &node.id,
                    NodeStatus::Failed,
                    failure,
                    b"",
                    stderr.as_bytes(),
                );
            }
        };

        let summary_json = serde_json::to_vec_pretty(&summary)?;
        exec.fs.write(&node_dir.join(OPERATION_SUMMARY_FILE), &summary_json)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        exec.fs.write(&stdout_path, &summary_json)?;
        exec.fs.write(&stderr_path, b"")?;

        let output_report = crate::inspect_declared_outputs(&outputs_dir, &node.outputs);
        if let Some(failure) = output_report.failure {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: output_report.output_evidence,
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: None,
            });
        }

        let fingerprint = crate::node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fingerprint, &output_report.present_outputs)?;

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
            failure: None,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        })
    }
}
