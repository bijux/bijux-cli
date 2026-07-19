use crate::{FailureInfo, MaterializeMode, PolicyConfig, RuntimeConfig, RuntimeError};
use bijux_dag_core::{Effect, Graph, Node, NodeKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdempotencyMode {
    Required,
    Recommended,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskIsolationMode {
    InProcess,
    Subprocess,
    Container,
    ExternalAdapter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SideEffectClassification {
    Required,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetryableFailureClass {
    ExecutionTransient,
    TimeoutTransient,
    ArtifactTransient,
    PolicyTransient,
    User,
    Infrastructure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeoutRetryPolicy {
    ByFailureClass,
    Always,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyV2 {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub backoff_ms: u64,
    pub jitter_ms: u64,
    pub timeout_retry_policy: TimeoutRetryPolicy,
    pub retryable_failure_classes: Vec<RetryableFailureClass>,
    pub retryable_exit_codes: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryFailureObservation {
    pub failure_class: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryDecision {
    pub retryable: bool,
    pub retry_allowed: bool,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    pub queue_timeout_ms: Option<u64>,
    pub execution_timeout_ms: Option<u64>,
    pub total_budget_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputMaterializationPolicy {
    Copy,
    Hardlink,
    Symlink,
    StoreOnly,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskFailureReason {
    Validation,
    Planner,
    Policy,
    Execution,
    Timeout,
    Cancellation,
    Artifact,
    Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInputDescriptor {
    pub name: String,
    pub value_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutputDescriptor {
    pub name: String,
    pub path: String,
    pub schema_name: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEffectDescriptor {
    pub effect: Effect,
    pub classification: SideEffectClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicyDescriptor {
    pub isolation_mode: TaskIsolationMode,
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContract {
    pub node_id: String,
    pub isolation_mode: TaskIsolationMode,
    pub inputs: Vec<TaskInputDescriptor>,
    pub outputs: Vec<TaskOutputDescriptor>,
    pub effects: Vec<TaskEffectDescriptor>,
    pub retry_policy: RetryPolicyV2,
    pub timeout_policy: TimeoutPolicy,
    pub idempotency_mode: IdempotencyMode,
    pub nondeterministic_allowed: bool,
    pub output_materialization_policy: OutputMaterializationPolicy,
    pub sandbox_policy: SandboxPolicyDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProvenance {
    pub executable_identity: String,
    pub adapter_identity: String,
    pub resolved_task_contract: TaskContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultEnvelope {
    pub node_id: String,
    pub status: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub attempts: u32,
    pub diagnostics: Vec<String>,
    pub effect_summary: Vec<String>,
    pub outputs: Vec<String>,
    pub provenance: NodeProvenance,
    pub failure_reason: Option<TaskFailureReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForcedCancellationCleanup {
    GracefulSignalThenTerminate,
    ImmediateTerminate,
    NoActionForInProcess,
}

#[derive(Debug, Clone)]
pub struct RuntimeState {
    cancellation_requested: Arc<AtomicBool>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self { cancellation_requested: Arc::new(AtomicBool::new(false)) }
    }

    pub fn request_cancellation(&self) {
        self.cancellation_requested.store(true, Ordering::SeqCst);
    }

    pub fn cancellation_requested(&self) -> bool {
        self.cancellation_requested.load(Ordering::SeqCst)
    }
}

fn parse_idempotency_mode(node: &Node) -> IdempotencyMode {
    let raw = param_literal_str(node, "idempotency_mode").unwrap_or("best_effort");
    match raw {
        "required" => IdempotencyMode::Required,
        "recommended" => IdempotencyMode::Recommended,
        _ => IdempotencyMode::BestEffort,
    }
}

fn parse_nondeterministic_allowed(node: &Node, graph_default: bool) -> bool {
    param_literal_bool(node, "nondeterministic_allowed").unwrap_or(graph_default)
}

fn parse_output_materialization_policy(
    node: &Node,
    runtime_materialize_mode: MaterializeMode,
) -> OutputMaterializationPolicy {
    let raw = param_literal_str(node, "output_materialization_policy");
    if let Some(raw) = raw {
        return match raw {
            "copy" => OutputMaterializationPolicy::Copy,
            "hardlink" => OutputMaterializationPolicy::Hardlink,
            "symlink" => OutputMaterializationPolicy::Symlink,
            "store_only" => OutputMaterializationPolicy::StoreOnly,
            "deferred" => OutputMaterializationPolicy::Deferred,
            _ => OutputMaterializationPolicy::Copy,
        };
    }
    match runtime_materialize_mode {
        MaterializeMode::Copy => OutputMaterializationPolicy::Copy,
        MaterializeMode::Hardlink => OutputMaterializationPolicy::Hardlink,
        MaterializeMode::Symlink => OutputMaterializationPolicy::Symlink,
    }
}

fn isolation_mode_for_node(node: &Node) -> TaskIsolationMode {
    match node.kind {
        NodeKind::Const => TaskIsolationMode::InProcess,
        NodeKind::Http => TaskIsolationMode::InProcess,
        NodeKind::FileTransform => TaskIsolationMode::InProcess,
        NodeKind::Shell => TaskIsolationMode::Subprocess,
        NodeKind::Python => TaskIsolationMode::Subprocess,
        NodeKind::Container => TaskIsolationMode::Container,
        NodeKind::External(_) => TaskIsolationMode::ExternalAdapter,
    }
}

pub fn build_retry_policy(node: &Node) -> RetryPolicyV2 {
    let strategy = match param_literal_str(node, "retry_backoff_strategy").unwrap_or("linear") {
        "fixed" => BackoffStrategy::Fixed,
        "exponential" => BackoffStrategy::Exponential,
        _ => BackoffStrategy::Linear,
    };
    let timeout_retry_policy =
        match param_literal_str(node, "timeout_retry_policy").unwrap_or("by_failure_class") {
            "always" => TimeoutRetryPolicy::Always,
            "never" => TimeoutRetryPolicy::Never,
            _ => TimeoutRetryPolicy::ByFailureClass,
        };
    let retryable_failure_classes = param_literal_string_vec(node, "retryable_failure_classes")
        .map(|values| {
            values
                .into_iter()
                .filter_map(|value| match value.as_str() {
                    "execution" | "execution_transient" => {
                        Some(RetryableFailureClass::ExecutionTransient)
                    }
                    "timeout" | "timeout_transient" => {
                        Some(RetryableFailureClass::TimeoutTransient)
                    }
                    "artifact_transient" => Some(RetryableFailureClass::ArtifactTransient),
                    "policy" | "policy_transient" => Some(RetryableFailureClass::PolicyTransient),
                    "user" => Some(RetryableFailureClass::User),
                    "infrastructure" => Some(RetryableFailureClass::Infrastructure),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| {
            vec![RetryableFailureClass::ExecutionTransient, RetryableFailureClass::TimeoutTransient]
        });
    let retryable_exit_codes =
        param_literal_i32_vec(node, "retryable_exit_codes").unwrap_or_default();
    RetryPolicyV2 {
        max_attempts: node.retry.max_attempts,
        backoff_strategy: strategy,
        backoff_ms: node.retry.backoff_ms,
        jitter_ms: param_literal_u64(node, "retry_jitter_ms").unwrap_or(0),
        timeout_retry_policy,
        retryable_failure_classes,
        retryable_exit_codes,
    }
}

pub fn retry_observation_from_failure(failure: &FailureInfo) -> RetryFailureObservation {
    RetryFailureObservation {
        failure_class: failure.operator_class().as_str().to_string(),
        failure_code: Some(failure.code.clone()),
        exit_code: failure
            .details
            .as_ref()
            .and_then(|details| details.get("exit_code"))
            .and_then(|value| value.as_i64())
            .and_then(|value| i32::try_from(value).ok()),
    }
}

pub fn retry_observation(
    failure_class: impl Into<String>,
    failure_code: Option<&str>,
    exit_code: Option<i32>,
) -> RetryFailureObservation {
    RetryFailureObservation {
        failure_class: failure_class.into(),
        failure_code: failure_code.map(ToString::to_string),
        exit_code,
    }
}

pub fn retry_backoff_ms(policy: &RetryPolicyV2, attempt: u32) -> u64 {
    if attempt == 0 {
        return 0;
    }
    match policy.backoff_strategy {
        BackoffStrategy::Fixed => policy.backoff_ms,
        BackoffStrategy::Linear => policy.backoff_ms.saturating_mul(attempt as u64),
        BackoffStrategy::Exponential => {
            let ordinal = attempt.saturating_sub(1) as u64;
            let multiplier = 1u64.checked_shl(ordinal.min(20) as u32).unwrap_or(u64::MAX);
            policy.backoff_ms.saturating_mul(multiplier)
        }
    }
}

pub fn retry_jitter_ms(node_id: &str, attempt: u32, failure_class: &str, jitter_ms: u64) -> u64 {
    if jitter_ms == 0 {
        return 0;
    }
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    node_id.hash(&mut hasher);
    attempt.hash(&mut hasher);
    failure_class.hash(&mut hasher);
    hasher.finish() % jitter_ms.saturating_add(1)
}

pub fn retry_wait_ms(
    node_id: &str,
    policy: &RetryPolicyV2,
    attempt: u32,
    failure_class: &str,
) -> u64 {
    retry_backoff_ms(policy, attempt).saturating_add(retry_jitter_ms(
        node_id,
        attempt,
        failure_class,
        policy.jitter_ms,
    ))
}

pub fn evaluate_retry_decision(
    node_id: &str,
    policy: &RetryPolicyV2,
    attempt: u32,
    observation: &RetryFailureObservation,
) -> RetryDecision {
    let normalized_class = normalize_retry_failure_class(&observation.failure_class);
    if normalized_class == "policy"
        || observation.failure_code.as_deref().is_some_and(|code| code.starts_with("POLICY_"))
    {
        return RetryDecision {
            retryable: false,
            retry_allowed: false,
            reason: "policy_failures_are_non_retryable".to_string(),
            matched_exit_code: None,
        };
    }

    let retryable_classes = policy
        .retryable_failure_classes
        .iter()
        .map(retryable_failure_class_name)
        .collect::<BTreeSet<_>>();
    let class_retryable = retryable_classes.contains(normalized_class.as_str());
    let matched_exit_code =
        observation.exit_code.filter(|exit_code| policy.retryable_exit_codes.contains(exit_code));

    let (retryable, reason) = if normalized_class == "timeout" {
        match policy.timeout_retry_policy {
            TimeoutRetryPolicy::Never => {
                (false, "timeout_retry_policy_denies_timeout_retry".to_string())
            }
            TimeoutRetryPolicy::Always => {
                (true, "timeout_retry_policy_allows_timeout_retry".to_string())
            }
            TimeoutRetryPolicy::ByFailureClass if matched_exit_code.is_some() => {
                (true, "retryable_exit_code_matched".to_string())
            }
            TimeoutRetryPolicy::ByFailureClass if class_retryable => {
                (true, "retryable_failure_class_matched".to_string())
            }
            TimeoutRetryPolicy::ByFailureClass => {
                (false, "timeout_failure_is_not_retry_eligible".to_string())
            }
        }
    } else if matched_exit_code.is_some() {
        (true, "retryable_exit_code_matched".to_string())
    } else if class_retryable {
        (true, "retryable_failure_class_matched".to_string())
    } else {
        (false, "failure_is_not_retry_eligible".to_string())
    };

    if !retryable {
        return RetryDecision { retryable, retry_allowed: false, reason, matched_exit_code };
    }

    if attempt > policy.max_attempts {
        return RetryDecision {
            retryable: true,
            retry_allowed: false,
            reason: "retry_budget_exhausted".to_string(),
            matched_exit_code,
        };
    }

    let _ = node_id;
    RetryDecision { retryable: true, retry_allowed: true, reason, matched_exit_code }
}

fn build_timeout_policy(node: &Node, options: &RuntimeConfig) -> TimeoutPolicy {
    TimeoutPolicy {
        queue_timeout_ms: param_literal_u64(node, "queue_timeout_ms"),
        execution_timeout_ms: param_literal_u64(node, "execution_timeout_ms")
            .or(node.timeout_ms)
            .or(options.node_timeout_ms),
        total_budget_timeout_ms: param_literal_u64(node, "total_budget_timeout_ms")
            .or(options.run_timeout_ms),
    }
}

fn classify_effects(node: &Node) -> Vec<TaskEffectDescriptor> {
    node.effects
        .iter()
        .cloned()
        .map(|effect| TaskEffectDescriptor {
            effect,
            classification: SideEffectClassification::Required,
        })
        .collect()
}

fn build_input_descriptors(node: &Node) -> Vec<TaskInputDescriptor> {
    node.inputs
        .iter()
        .map(|name| TaskInputDescriptor {
            name: name.clone(),
            value_type: "artifact_ref".to_string(),
            required: true,
        })
        .collect()
}

fn build_output_descriptors(node: &Node) -> Vec<TaskOutputDescriptor> {
    node.outputs
        .iter()
        .map(|out| TaskOutputDescriptor {
            name: out.name.clone(),
            path: out.path.clone(),
            schema_name: "bijux.output.file".to_string(),
            schema_version: "v0.1".to_string(),
        })
        .collect()
}

fn build_sandbox_policy(
    node: &Node,
    policy: &PolicyConfig,
    isolation_mode: TaskIsolationMode,
) -> SandboxPolicyDescriptor {
    let node_uses_network = node.effects.contains(&Effect::Network);
    let node_uses_env = node.effects.contains(&Effect::Env);
    let node_uses_clock = node.effects.contains(&Effect::Clock);
    SandboxPolicyDescriptor {
        isolation_mode,
        deny_network: policy.deny_network || !node_uses_network,
        deny_env: policy.deny_env || !node_uses_env,
        deny_clock: policy.deny_clock || !node_uses_clock,
        clean_env: policy.clean_env,
    }
}

pub fn build_task_contract(node: &Node, graph: &Graph, options: &RuntimeConfig) -> TaskContract {
    let isolation_mode = isolation_mode_for_node(node);
    TaskContract {
        node_id: node.id.clone(),
        isolation_mode: isolation_mode.clone(),
        inputs: build_input_descriptors(node),
        outputs: build_output_descriptors(node),
        effects: classify_effects(node),
        retry_policy: build_retry_policy(node),
        timeout_policy: build_timeout_policy(node, options),
        idempotency_mode: parse_idempotency_mode(node),
        nondeterministic_allowed: parse_nondeterministic_allowed(
            node,
            graph.nondeterminism_allowed,
        ),
        output_materialization_policy: parse_output_materialization_policy(
            node,
            options.materialize_inputs,
        ),
        sandbox_policy: build_sandbox_policy(node, &options.policy, isolation_mode),
    }
}

pub fn validate_task_contracts(
    graph: &Graph,
    options: &RuntimeConfig,
) -> Result<Vec<TaskContract>, RuntimeError> {
    let mut contracts = Vec::new();
    for node in &graph.nodes {
        let contract = build_task_contract(node, graph, options);

        if !node.env_allowlist.is_empty() && !node.effects.contains(&Effect::Env) {
            return Err(RuntimeError::Executor(format!(
                "node '{}' declares env_allowlist without env effect",
                node.id
            )));
        }
        if let Some(resources) = node.resources.as_ref() {
            if resources.cpu == 0 {
                return Err(RuntimeError::Executor(format!(
                    "node '{}' has invalid resource contract; cpu must be positive",
                    node.id
                )));
            }
            for (name, amount) in &resources.named_resources {
                if name.trim().is_empty() {
                    return Err(RuntimeError::Executor(format!(
                        "node '{}' has invalid resource contract; named resource must not be empty",
                        node.id
                    )));
                }
                if *amount == 0 {
                    return Err(RuntimeError::Executor(format!(
                        "node '{}' has invalid resource contract; named resource '{}' must be positive",
                        node.id, name
                    )));
                }
            }
        }
        contracts.push(contract);
    }
    Ok(contracts)
}

pub fn default_forced_cleanup(mode: &TaskIsolationMode) -> ForcedCancellationCleanup {
    match mode {
        TaskIsolationMode::InProcess => ForcedCancellationCleanup::NoActionForInProcess,
        TaskIsolationMode::Subprocess => ForcedCancellationCleanup::GracefulSignalThenTerminate,
        TaskIsolationMode::Container | TaskIsolationMode::ExternalAdapter => {
            ForcedCancellationCleanup::ImmediateTerminate
        }
    }
}

fn param_literal_str<'a>(node: &'a Node, key: &str) -> Option<&'a str> {
    match &node.params {
        bijux_dag_core::ParamValue::Object(map) => match map.get(key) {
            Some(bijux_dag_core::ParamValue::Literal(value)) => value.as_str(),
            _ => None,
        },
        _ => None,
    }
}

fn param_literal_bool(node: &Node, key: &str) -> Option<bool> {
    match &node.params {
        bijux_dag_core::ParamValue::Object(map) => match map.get(key) {
            Some(bijux_dag_core::ParamValue::Literal(value)) => value.as_bool(),
            _ => None,
        },
        _ => None,
    }
}

fn param_literal_u64(node: &Node, key: &str) -> Option<u64> {
    match &node.params {
        bijux_dag_core::ParamValue::Object(map) => match map.get(key) {
            Some(bijux_dag_core::ParamValue::Literal(value)) => value.as_u64(),
            _ => None,
        },
        _ => None,
    }
}

fn param_literal_string_vec(node: &Node, key: &str) -> Option<Vec<String>> {
    match &node.params {
        bijux_dag_core::ParamValue::Object(map) => match map.get(key) {
            Some(bijux_dag_core::ParamValue::Literal(value)) => value.as_array().map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            }),
            Some(bijux_dag_core::ParamValue::Array(items)) => Some(
                items
                    .iter()
                    .filter_map(|item| match item {
                        bijux_dag_core::ParamValue::Literal(value) => {
                            value.as_str().map(ToString::to_string)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        },
        _ => None,
    }
}

fn param_literal_i32_vec(node: &Node, key: &str) -> Option<Vec<i32>> {
    match &node.params {
        bijux_dag_core::ParamValue::Object(map) => match map.get(key) {
            Some(bijux_dag_core::ParamValue::Literal(value)) => value.as_array().map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_i64().and_then(|value| i32::try_from(value).ok()))
                    .collect::<Vec<_>>()
            }),
            Some(bijux_dag_core::ParamValue::Array(items)) => Some(
                items
                    .iter()
                    .filter_map(|item| match item {
                        bijux_dag_core::ParamValue::Literal(value) => {
                            value.as_i64().and_then(|value| i32::try_from(value).ok())
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        },
        _ => None,
    }
}

fn normalize_retry_failure_class(value: &str) -> String {
    let normalized = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect::<String>();
    match normalized.as_str() {
        "execution" | "executiontransient" => "execution".to_string(),
        "timeout" | "timeouttransient" => "timeout".to_string(),
        "artifact" | "artifacttransient" => "artifact".to_string(),
        "policy" | "policytransient" => "policy".to_string(),
        "user" => "user".to_string(),
        "infrastructure" => "infrastructure".to_string(),
        _ => normalized,
    }
}

fn retryable_failure_class_name(value: &RetryableFailureClass) -> &'static str {
    match value {
        RetryableFailureClass::ExecutionTransient => "execution",
        RetryableFailureClass::TimeoutTransient => "timeout",
        RetryableFailureClass::ArtifactTransient => "artifact",
        RetryableFailureClass::PolicyTransient => "policy",
        RetryableFailureClass::User => "user",
        RetryableFailureClass::Infrastructure => "infrastructure",
    }
}
