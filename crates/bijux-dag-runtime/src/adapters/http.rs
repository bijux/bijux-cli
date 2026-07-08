use crate::{
    Adapter, AdapterId, FailureClass, FailureInfo, NodeCtx, NodeResult, NodeStatus, RuntimeError,
};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use bijux_dag_artifacts::write_outputs_index;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{Method, Url};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

const FAILURE_BODY_PREVIEW_BYTES: usize = 4096;

#[derive(Clone)]
pub struct HttpRequestAdapter;

struct HttpRequestParams {
    method: Method,
    method_text: String,
    url: Url,
    headers: BTreeMap<String, String>,
    body: Option<Value>,
}

#[derive(Debug, Serialize)]
struct HttpResponseArtifact {
    request: HttpRequestArtifact,
    response: HttpResponsePayload,
}

#[derive(Debug, Serialize)]
struct HttpRequestArtifact {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<HttpBodyArtifact>,
}

#[derive(Debug, Serialize)]
struct HttpResponsePayload {
    status: u16,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    headers: BTreeMap<String, Vec<String>>,
    body: HttpBodyArtifact,
}

#[derive(Debug, Clone, Serialize)]
struct HttpBodyArtifact {
    bytes: usize,
    sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base64: Option<String>,
}

fn http_user_failure(message: impl Into<String>, details: Option<Value>) -> FailureInfo {
    FailureInfo::new(FailureClass::User, "User", "EXEC_ERROR", message.into(), details)
}

fn http_execution_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<Value>,
) -> FailureInfo {
    FailureInfo::new(FailureClass::Execution, "Execution", code.into(), message.into(), details)
}

fn node_failure_result(
    fs: &dyn crate::Fs,
    stdout_path: &Path,
    stderr_path: &Path,
    outputs_dir: &Path,
    status: NodeStatus,
    failure: FailureInfo,
    stderr_contents: &[u8],
) -> Result<NodeResult, RuntimeError> {
    fs.write(stdout_path, b"")?;
    fs.write(stderr_path, stderr_contents)?;
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

fn complete_node_result(
    ctx: &NodeCtx,
    stdout_path: &Path,
    stderr_path: &Path,
    outputs_dir: &Path,
    status: NodeStatus,
    failure: Option<FailureInfo>,
    stderr_contents: &[u8],
) -> Result<NodeResult, RuntimeError> {
    let exec = ctx.exec;
    exec.fs.write(stdout_path, b"")?;
    exec.fs.write(stderr_path, stderr_contents)?;

    let output_report = crate::inspect_declared_outputs(outputs_dir, &ctx.node.outputs);
    if let Some(output_failure) = output_report.failure {
        return Ok(NodeResult {
            status: NodeStatus::Failed,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
            failure: Some(output_failure),
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        });
    }

    let fingerprint = crate::node_fingerprint_from_ctx(exec, &ctx.node.id);
    write_outputs_index(outputs_dir, &ctx.node.id, &fingerprint, &output_report.present_outputs)?;

    Ok(NodeResult {
        status,
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: outputs_dir.display().to_string(),
        output_evidence: output_report.output_evidence,
        failure,
        attempts: 1,
        attempt_events: Vec::new(),
        container_meta: None,
        adapter_binary_sha256: None,
    })
}

fn parse_headers(
    params: &serde_json::Map<String, Value>,
) -> Result<BTreeMap<String, String>, FailureInfo> {
    let Some(headers) = params.get("headers") else {
        return Ok(BTreeMap::new());
    };
    let Some(headers) = headers.as_object() else {
        return Err(http_user_failure(
            "http headers must be an object of string values",
            Some(json!({
                "field": "headers",
                "reason": "expected_object",
            })),
        ));
    };

    let mut normalized = BTreeMap::new();
    for (name, value) in headers {
        let Some(value) = value.as_str() else {
            return Err(http_user_failure(
                "http headers must be an object of string values",
                Some(json!({
                    "field": "headers",
                    "header": name,
                    "reason": "expected_string",
                })),
            ));
        };
        HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
            http_user_failure(
                "http header name is invalid",
                Some(json!({
                    "field": "headers",
                    "header": name,
                    "reason": "invalid_name",
                    "details": error.to_string(),
                })),
            )
        })?;
        HeaderValue::from_str(value).map_err(|error| {
            http_user_failure(
                "http header value is invalid",
                Some(json!({
                    "field": "headers",
                    "header": name,
                    "reason": "invalid_value",
                    "details": error.to_string(),
                })),
            )
        })?;
        normalized.insert(name.clone(), value.to_string());
    }
    Ok(normalized)
}

fn parse_http_params(params: &Value) -> Result<HttpRequestParams, FailureInfo> {
    let Some(params) = params.as_object() else {
        return Err(http_user_failure(
            "http params must be an object",
            Some(json!({
                "field": "params",
                "reason": "expected_object",
            })),
        ));
    };

    let method_text = match params.get("method") {
        Some(Value::String(method)) if !method.trim().is_empty() => {
            method.trim().to_ascii_uppercase()
        }
        Some(Value::String(_)) => {
            return Err(http_user_failure(
                "http method must not be empty",
                Some(json!({
                    "field": "method",
                    "reason": "empty",
                })),
            ));
        }
        Some(_) => {
            return Err(http_user_failure(
                "http method must be a string",
                Some(json!({
                    "field": "method",
                    "reason": "expected_string",
                })),
            ));
        }
        None => {
            return Err(http_user_failure(
                "http method is required",
                Some(json!({
                    "field": "method",
                    "reason": "missing",
                })),
            ));
        }
    };
    let method = Method::from_bytes(method_text.as_bytes()).map_err(|error| {
        http_user_failure(
            "http method is invalid",
            Some(json!({
                "field": "method",
                "reason": "invalid_method",
                "details": error.to_string(),
            })),
        )
    })?;

    let url_text = match params.get("url") {
        Some(Value::String(url)) if !url.trim().is_empty() => url.trim().to_string(),
        Some(Value::String(_)) => {
            return Err(http_user_failure(
                "http url must not be empty",
                Some(json!({
                    "field": "url",
                    "reason": "empty",
                })),
            ));
        }
        Some(_) => {
            return Err(http_user_failure(
                "http url must be a string",
                Some(json!({
                    "field": "url",
                    "reason": "expected_string",
                })),
            ));
        }
        None => {
            return Err(http_user_failure(
                "http url is required",
                Some(json!({
                    "field": "url",
                    "reason": "missing",
                })),
            ));
        }
    };
    let url = Url::parse(&url_text).map_err(|error| {
        http_user_failure(
            "http url is invalid",
            Some(json!({
                "field": "url",
                "reason": "invalid_url",
                "details": error.to_string(),
            })),
        )
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(http_user_failure(
            "http url must use http or https",
            Some(json!({
                "field": "url",
                "reason": "unsupported_scheme",
                "scheme": url.scheme(),
            })),
        ));
    }

    let headers = parse_headers(params)?;
    let body = params.get("body").cloned();

    Ok(HttpRequestParams { method, method_text, url, headers, body })
}

fn single_output_target(
    node: &bijux_dag_core::Node,
    outputs_dir: &Path,
) -> Result<PathBuf, FailureInfo> {
    if node.outputs.len() != 1 {
        return Err(http_user_failure(
            "http adapter requires exactly one declared output",
            Some(json!({
                "declared_outputs": node.outputs.len(),
            })),
        ));
    }

    let output = &node.outputs[0];
    if output.expects_directory() {
        return Err(FailureInfo::new(
            FailureClass::User,
            "User",
            "OUTPUT_PATH_INVALID",
            format!("http adapter cannot materialize directory output: {}", output.path),
            Some(json!({
                "output": output.name,
                "path": output.path,
            })),
        ));
    }
    crate::authorized_declared_output_path(outputs_dir, output)
}

fn body_artifact(bytes: &[u8], content_type: Option<&str>) -> HttpBodyArtifact {
    let sha256 = hex::encode(Sha256::digest(bytes));
    let text = String::from_utf8(bytes.to_vec()).ok();
    let json_value = text.as_ref().and_then(|body| serde_json::from_str::<Value>(body).ok());
    let base64 = text.is_none().then(|| BASE64_STANDARD.encode(bytes));

    HttpBodyArtifact {
        bytes: bytes.len(),
        sha256,
        content_type: content_type.map(ToString::to_string),
        text,
        json: json_value,
        base64,
    }
}

fn request_body_bytes(
    body: Option<&Value>,
    request_headers: &BTreeMap<String, String>,
) -> Result<(Option<Vec<u8>>, Option<HttpBodyArtifact>, bool), FailureInfo> {
    let Some(body) = body else {
        return Ok((None, None, false));
    };

    let body_bytes = match body {
        Value::String(text) => text.as_bytes().to_vec(),
        _ => serde_json::to_vec(body).map_err(|error| {
            http_user_failure(
                "http body could not be serialized",
                Some(json!({
                    "field": "body",
                    "reason": "json_serialize_failed",
                    "details": error.to_string(),
                })),
            )
        })?,
    };
    let content_type = request_headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.as_str());
    let add_default_json_content_type = !matches!(body, Value::String(_)) && content_type.is_none();
    let body_artifact = body_artifact(
        &body_bytes,
        if add_default_json_content_type { Some("application/json") } else { content_type },
    );
    Ok((Some(body_bytes), Some(body_artifact), add_default_json_content_type))
}

fn response_headers(headers: &HeaderMap) -> BTreeMap<String, Vec<String>> {
    let mut normalized = BTreeMap::new();
    for (name, value) in headers {
        if let Ok(text) = value.to_str() {
            normalized
                .entry(name.as_str().to_string())
                .or_insert_with(Vec::new)
                .push(text.to_string());
        }
    }
    normalized
}

fn failure_body_preview(body: &HttpBodyArtifact) -> Value {
    json!({
        "bytes": body.bytes,
        "sha256": body.sha256,
        "content_type": body.content_type,
        "text": body
            .text
            .as_ref()
            .map(|text| text.chars().take(FAILURE_BODY_PREVIEW_BYTES).collect::<String>()),
        "base64": body
            .base64
            .as_ref()
            .map(|encoded| encoded.chars().take(FAILURE_BODY_PREVIEW_BYTES).collect::<String>()),
        "json": body.json,
    })
}

impl Adapter for HttpRequestAdapter {
    fn id(&self) -> AdapterId {
        AdapterId { id: "http".to_string(), version: "0.1".to_string() }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["http".to_string()]
    }

    fn required_effects(&self) -> crate::EffectSet {
        crate::EffectSet { filesystem: true, env: false, network: true, clock: false }
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
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        if let Err(failure) = crate::preflight_declared_output_targets(&outputs_dir, &node.outputs) {
            let stderr_message = failure.message.clone();
            return node_failure_result(
                exec.fs.as_ref(),
                &stdout_path,
                &stderr_path,
                &outputs_dir,
                NodeStatus::Failed,
                failure,
                stderr_message.as_bytes(),
            );
        }

        let request = match parse_http_params(ctx.params) {
            Ok(request) => request,
            Err(failure) => {
                let stderr_message = failure.message.clone();
                return node_failure_result(
                    exec.fs.as_ref(),
                    &stdout_path,
                    &stderr_path,
                    &outputs_dir,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };
        let response_output = match single_output_target(node, &outputs_dir) {
            Ok(output) => output,
            Err(failure) => {
                let stderr_message = failure.message.clone();
                return node_failure_result(
                    exec.fs.as_ref(),
                    &stdout_path,
                    &stderr_path,
                    &outputs_dir,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };

        let timeout_ms = crate::effective_node_timeout_ms(node, ctx.params);
        let mut client_builder = Client::builder();
        if let Some(timeout_ms) = timeout_ms {
            client_builder = client_builder.timeout(Duration::from_millis(timeout_ms));
        }
        let client = client_builder.build().map_err(|error| {
            RuntimeError::Executor(format!("http client initialization failed: {error}"))
        })?;

        let (request_body_bytes, request_body, add_default_json_content_type) =
            match request_body_bytes(request.body.as_ref(), &request.headers) {
                Ok(result) => result,
                Err(failure) => {
                    let stderr_message = failure.message.clone();
                    return node_failure_result(
                        exec.fs.as_ref(),
                        &stdout_path,
                        &stderr_path,
                        &outputs_dir,
                        NodeStatus::Failed,
                        failure,
                        stderr_message.as_bytes(),
                    );
                }
            };

        let mut request_builder = client.request(request.method.clone(), request.url.clone());
        for (name, value) in &request.headers {
            request_builder = request_builder.header(name, value);
        }
        if add_default_json_content_type {
            request_builder = request_builder.header(CONTENT_TYPE, "application/json");
        }
        if let Some(body_bytes) = request_body_bytes {
            request_builder = request_builder.body(body_bytes);
        }

        let response = match request_builder.send() {
            Ok(response) => response,
            Err(error) if error.is_timeout() => {
                return node_failure_result(
                    exec.fs.as_ref(),
                    &stdout_path,
                    &stderr_path,
                    &outputs_dir,
                    NodeStatus::Failed,
                    FailureInfo::new(
                        FailureClass::Timeout,
                        "Timeout",
                        "EXEC_TIMEOUT",
                        "http request timed out after configured node timeout",
                        Some(json!({
                            "method": request.method_text,
                            "url": request.url.as_str(),
                        })),
                    ),
                    b"http request timed out after configured node timeout",
                );
            }
            Err(error) => {
                let failure = http_execution_failure(
                    "HTTP_REQUEST_ERROR",
                    "http request failed before a response was received",
                    Some(json!({
                        "method": request.method_text,
                        "url": request.url.as_str(),
                        "details": error.to_string(),
                    })),
                );
                let stderr_message = failure.message.clone();
                return node_failure_result(
                    exec.fs.as_ref(),
                    &stdout_path,
                    &stderr_path,
                    &outputs_dir,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };

        let status = response.status();
        let reason = status.canonical_reason().map(ToString::to_string);
        let response_headers = response_headers(response.headers());
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let response_bytes = response.bytes().map_err(|error| {
            RuntimeError::Executor(format!("http response body read failed: {error}"))
        })?;
        let response_body = body_artifact(&response_bytes, content_type.as_deref());

        let artifact = HttpResponseArtifact {
            request: HttpRequestArtifact {
                method: request.method_text.clone(),
                url: request.url.to_string(),
                headers: request.headers.clone(),
                body: request_body,
            },
            response: HttpResponsePayload {
                status: status.as_u16(),
                success: status.is_success(),
                reason: reason.clone(),
                headers: response_headers,
                body: response_body.clone(),
            },
        };
        if let Some(parent) = response_output.parent() {
            exec.fs.create_dir_all(parent)?;
        }
        exec.fs.write(&response_output, &serde_json::to_vec_pretty(&artifact)?)?;

        if !status.is_success() {
            let failure = http_execution_failure(
                "HTTP_STATUS_ERROR",
                format!("http request returned status {}", status.as_u16()),
                Some(json!({
                    "method": request.method_text,
                    "url": request.url.as_str(),
                    "status": status.as_u16(),
                    "reason": reason,
                    "response_body": failure_body_preview(&response_body),
                })),
            );
            return complete_node_result(
                ctx,
                &stdout_path,
                &stderr_path,
                &outputs_dir,
                NodeStatus::Failed,
                Some(failure),
                format!("http request returned status {}", status.as_u16()).as_bytes(),
            );
        }

        complete_node_result(
            ctx,
            &stdout_path,
            &stderr_path,
            &outputs_dir,
            NodeStatus::Success,
            None,
            b"",
        )
    }
}
