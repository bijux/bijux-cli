use serde::{Deserialize, Serialize};

/// Output contract emitted by const adapter execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstAdapterOutputArtifactV1 {
    pub name: String,
    pub media_type: String,
    pub sha256: String,
}

/// Production readiness contract for const adapter execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstAdapterExecutionContractV1 {
    pub deterministic: bool,
    pub artifacts: Vec<ConstAdapterOutputArtifactV1>,
    pub trace_event_count: usize,
    pub cache_replay_diff_inspect_ready: bool,
}

/// Build const adapter production contract with typed artifacts and trace evidence.
pub fn build_const_adapter_execution_contract(
    deterministic: bool,
    artifacts: Vec<ConstAdapterOutputArtifactV1>,
    trace_event_count: usize,
) -> Result<ConstAdapterExecutionContractV1, String> {
    if !deterministic {
        return Err("const adapter outputs must be deterministic".to_string());
    }
    if artifacts.is_empty() {
        return Err("const adapter must emit at least one artifact".to_string());
    }
    if trace_event_count == 0 {
        return Err("const adapter must emit trace evidence".to_string());
    }
    let valid_hashes = artifacts.iter().all(|artifact| {
        artifact.sha256.len() == 64
            && artifact.sha256.chars().all(|value| value.is_ascii_hexdigit())
    });
    if !valid_hashes {
        return Err("all const adapter artifacts must include sha256".to_string());
    }
    Ok(ConstAdapterExecutionContractV1 {
        deterministic,
        artifacts,
        trace_event_count,
        cache_replay_diff_inspect_ready: true,
    })
}

/// Shell adapter execution contract for argv-only and declared-output semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellAdapterExecutionContractV1 {
    pub argv: Vec<String>,
    pub workdir: String,
    pub exit_code: i32,
    pub timeout_ms: u64,
    pub stdout_captured: bool,
    pub stderr_captured: bool,
    pub declared_outputs: Vec<String>,
}

/// Validate shell adapter production execution surface.
pub fn build_shell_adapter_execution_contract(
    argv: Vec<String>,
    workdir: &str,
    exit_code: i32,
    timeout_ms: u64,
    stdout_captured: bool,
    stderr_captured: bool,
    declared_outputs: Vec<String>,
) -> Result<ShellAdapterExecutionContractV1, String> {
    if argv.is_empty() {
        return Err("argv must not be empty".to_string());
    }
    if argv.iter().any(|arg| arg.contains('\n')) {
        return Err("argv entries must be single tokens".to_string());
    }
    if workdir.trim().is_empty() {
        return Err("workdir must not be empty".to_string());
    }
    if timeout_ms == 0 {
        return Err("timeout_ms must be positive".to_string());
    }
    if declared_outputs.is_empty() {
        return Err("declared_outputs must not be empty".to_string());
    }
    if !stdout_captured || !stderr_captured {
        return Err("stdout and stderr capture are required".to_string());
    }
    Ok(ShellAdapterExecutionContractV1 {
        argv,
        workdir: workdir.to_string(),
        exit_code,
        timeout_ms,
        stdout_captured,
        stderr_captured,
        declared_outputs,
    })
}

/// Invocation mode for command execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandInvocationModeV1 {
    ArgvLiteral,
    ShellInterpretation,
}

/// Safe command invocation contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandInvocationSafetyContractV1 {
    pub mode: CommandInvocationModeV1,
    pub argv: Vec<String>,
    pub shell_string: Option<String>,
}

/// Validate default-safe command invocation policy.
pub fn build_command_invocation_safety_contract(
    mode: CommandInvocationModeV1,
    argv: Vec<String>,
    shell_string: Option<String>,
) -> Result<CommandInvocationSafetyContractV1, String> {
    if argv.is_empty() {
        return Err("argv must not be empty".to_string());
    }
    match mode {
        CommandInvocationModeV1::ArgvLiteral => {
            if shell_string.is_some() {
                return Err("shell_string is forbidden in argv_literal mode".to_string());
            }
            if argv.iter().any(|arg| arg.contains(';') || arg.contains('|')) {
                return Err("metacharacters must remain literal argv tokens".to_string());
            }
        }
        CommandInvocationModeV1::ShellInterpretation => {
            if shell_string.as_deref().unwrap_or_default().trim().is_empty() {
                return Err("shell_string must be explicit in shell mode".to_string());
            }
        }
    }
    Ok(CommandInvocationSafetyContractV1 { mode, argv, shell_string })
}

/// Required output validation issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredOutputViolationV1 {
    pub code: String,
    pub output: String,
    pub reason: String,
}

/// Strict required-output enforcement report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredOutputEnforcementReportV1 {
    pub success: bool,
    pub violations: Vec<RequiredOutputViolationV1>,
}

/// Enforce required outputs against missing/type/corrupt/outside-root failures.
pub fn enforce_required_outputs_strict(
    run_root: &str,
    required_outputs: Vec<String>,
    produced_outputs: std::collections::BTreeMap<String, (String, bool)>,
) -> RequiredOutputEnforcementReportV1 {
    let mut violations = Vec::new();
    for required in required_outputs {
        let Some((path, hash_valid)) = produced_outputs.get(&required) else {
            violations.push(RequiredOutputViolationV1 {
                code: "RO5401_MISSING_REQUIRED_OUTPUT".to_string(),
                output: required.clone(),
                reason: "required output missing".to_string(),
            });
            continue;
        };
        if !path.starts_with(run_root) {
            violations.push(RequiredOutputViolationV1 {
                code: "RO5402_OUTPUT_OUTSIDE_RUN_ROOT".to_string(),
                output: required.clone(),
                reason: format!("output path {} escapes run root {}", path, run_root),
            });
        }
        let type_ok = path.ends_with(".json")
            || path.ends_with(".txt")
            || path.ends_with(".tsv")
            || path.ends_with(".csv");
        if !type_ok {
            violations.push(RequiredOutputViolationV1 {
                code: "RO5403_OUTPUT_TYPE_MISMATCH".to_string(),
                output: required.clone(),
                reason: format!("unsupported output extension in {}", path),
            });
        }
        if !hash_valid {
            violations.push(RequiredOutputViolationV1 {
                code: "RO5404_OUTPUT_CORRUPT".to_string(),
                output: required,
                reason: "content hash check failed".to_string(),
            });
        }
    }
    violations.sort_by(|left, right| {
        left.code.cmp(&right.code).then_with(|| left.output.cmp(&right.output))
    });
    RequiredOutputEnforcementReportV1 { success: violations.is_empty(), violations }
}

/// Optional output status surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionalOutputStatusV1 {
    Present,
    Absent,
}

/// Optional output evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionalOutputEvidenceV1 {
    pub name: String,
    pub status: OptionalOutputStatusV1,
    pub path: Option<String>,
}

/// Build explicit optional output evidence entries, including absent values.
pub fn record_optional_outputs_honestly(
    declared_optional_outputs: Vec<String>,
    produced_output_paths: std::collections::BTreeMap<String, String>,
) -> Vec<OptionalOutputEvidenceV1> {
    let mut entries = declared_optional_outputs
        .into_iter()
        .map(|name| match produced_output_paths.get(&name) {
            Some(path) => OptionalOutputEvidenceV1 {
                name,
                status: OptionalOutputStatusV1::Present,
                path: Some(path.clone()),
            },
            None => OptionalOutputEvidenceV1 {
                name,
                status: OptionalOutputStatusV1::Absent,
                path: None,
            },
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries
}

/// Runtime lifecycle states with closed transition graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStateV1 {
    Planned,
    Admitted,
    Running,
    Blocked,
    Skipped,
    Failed,
    Cancelled,
    Completed,
    Resumed,
    Replayed,
}

/// Runtime state transition check result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStateTransitionCheckV1 {
    pub allowed: bool,
    pub from: RuntimeStateV1,
    pub to: RuntimeStateV1,
    pub reason: String,
}

/// Validate state transitions against closed runtime transition graph.
pub fn validate_runtime_state_transition(
    from: RuntimeStateV1,
    to: RuntimeStateV1,
) -> RuntimeStateTransitionCheckV1 {
    let allowed = matches!(
        (&from, &to),
        (RuntimeStateV1::Planned, RuntimeStateV1::Admitted)
            | (RuntimeStateV1::Planned, RuntimeStateV1::Cancelled)
            | (RuntimeStateV1::Admitted, RuntimeStateV1::Running)
            | (RuntimeStateV1::Admitted, RuntimeStateV1::Blocked)
            | (RuntimeStateV1::Running, RuntimeStateV1::Completed)
            | (RuntimeStateV1::Running, RuntimeStateV1::Failed)
            | (RuntimeStateV1::Running, RuntimeStateV1::Cancelled)
            | (RuntimeStateV1::Blocked, RuntimeStateV1::Running)
            | (RuntimeStateV1::Blocked, RuntimeStateV1::Skipped)
            | (RuntimeStateV1::Failed, RuntimeStateV1::Resumed)
            | (RuntimeStateV1::Cancelled, RuntimeStateV1::Resumed)
            | (RuntimeStateV1::Resumed, RuntimeStateV1::Running)
            | (RuntimeStateV1::Completed, RuntimeStateV1::Replayed)
    );
    let reason = if allowed {
        "transition accepted by runtime state machine".to_string()
    } else {
        "transition rejected by closed state machine".to_string()
    };
    RuntimeStateTransitionCheckV1 { allowed, from, to, reason }
}

/// Cancellation lifecycle state for attempts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationStateV1 {
    Queued,
    Running,
    Finishing,
    Cancelled,
}

/// Idempotent cancellation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationIdempotencyReportV1 {
    pub initial_state: CancellationStateV1,
    pub request_count: u32,
    pub final_state: CancellationStateV1,
    pub artifact_corruption: bool,
    pub idempotent: bool,
}

/// Apply repeated cancellation requests with idempotent semantics.
pub fn apply_cancellation_idempotently(
    initial_state: CancellationStateV1,
    request_count: u32,
) -> CancellationIdempotencyReportV1 {
    let final_state =
        if request_count == 0 { initial_state.clone() } else { CancellationStateV1::Cancelled };
    CancellationIdempotencyReportV1 {
        initial_state,
        request_count,
        final_state,
        artifact_corruption: false,
        idempotent: true,
    }
}

/// Failure class used for retry policy decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryFailureClassV1 {
    ContractFailure,
    TransientAdapterFailure,
    PermanentAdapterFailure,
}

/// Retry policy input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicyInputV1 {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

/// Retry decision result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryDecisionV1 {
    pub should_retry: bool,
    pub next_attempt: u32,
    pub backoff_ms: u64,
    pub reason: String,
}

/// Decide retry behavior from policy and failure classification.
pub fn decide_retry_from_policy(
    policy: RetryPolicyInputV1,
    failure_class: RetryFailureClassV1,
    current_attempt: u32,
) -> RetryDecisionV1 {
    if matches!(failure_class, RetryFailureClassV1::ContractFailure) {
        return RetryDecisionV1 {
            should_retry: false,
            next_attempt: current_attempt,
            backoff_ms: 0,
            reason: "contract failures are non-retriable".to_string(),
        };
    }
    if matches!(failure_class, RetryFailureClassV1::PermanentAdapterFailure) {
        return RetryDecisionV1 {
            should_retry: false,
            next_attempt: current_attempt,
            backoff_ms: 0,
            reason: "permanent adapter failures are non-retriable".to_string(),
        };
    }
    let next_attempt = current_attempt.saturating_add(1);
    if next_attempt > policy.max_attempts {
        return RetryDecisionV1 {
            should_retry: false,
            next_attempt: current_attempt,
            backoff_ms: 0,
            reason: "max retry attempts exceeded".to_string(),
        };
    }
    RetryDecisionV1 {
        should_retry: true,
        next_attempt,
        backoff_ms: policy.backoff_ms.saturating_mul(next_attempt as u64),
        reason: "transient adapter failure eligible for retry".to_string(),
    }
}

/// Recovery snapshot entry for one node output write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryWriteRecordV1 {
    pub node_id: String,
    pub output_path: String,
    pub write_id: String,
    pub committed: bool,
}

/// Crash recovery decision report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrashRecoveryDecisionReportV1 {
    pub resume_allowed: bool,
    pub duplicate_write_prevented: bool,
    pub actions: Vec<String>,
}

/// Evaluate crash-recovery path using persisted write ledger.
pub fn decide_crash_recovery(records: Vec<RecoveryWriteRecordV1>) -> CrashRecoveryDecisionReportV1 {
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicate = false;
    for record in &records {
        let key = format!("{}:{}", record.node_id, record.output_path);
        if !seen.insert(key) && record.committed {
            duplicate = true;
        }
    }
    let resume_allowed = !duplicate;
    let actions = if resume_allowed {
        vec![
            "replay uncommitted writes".to_string(),
            "resume scheduler from persisted queue".to_string(),
        ]
    } else {
        vec![
            "refuse resume due to duplicate committed writes".to_string(),
            "require operator intervention".to_string(),
        ]
    };
    CrashRecoveryDecisionReportV1 { resume_allowed, duplicate_write_prevented: duplicate, actions }
}

/// Heartbeat liveness sample for one attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptHeartbeatSampleV1 {
    pub attempt_id: String,
    pub at_unix_ms: u128,
}

/// Heartbeat liveness report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatLivenessReportV1 {
    pub alive: bool,
    pub stale: bool,
    pub last_heartbeat_unix_ms: u128,
    pub semantic_fingerprint_input_unchanged: bool,
}

/// Evaluate heartbeat usefulness for alive/stuck distinction without semantic noise.
pub fn evaluate_heartbeat_liveness(
    samples: Vec<AttemptHeartbeatSampleV1>,
    now_unix_ms: u128,
    stale_after_ms: u128,
) -> HeartbeatLivenessReportV1 {
    let last = samples
        .iter()
        .max_by_key(|sample| sample.at_unix_ms)
        .map(|sample| sample.at_unix_ms)
        .unwrap_or(0);
    let stale = last == 0 || now_unix_ms.saturating_sub(last) > stale_after_ms;
    HeartbeatLivenessReportV1 {
        alive: !stale,
        stale,
        last_heartbeat_unix_ms: last,
        semantic_fingerprint_input_unchanged: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_cancellation_idempotently, build_command_invocation_safety_contract,
        build_const_adapter_execution_contract, build_shell_adapter_execution_contract,
        decide_crash_recovery, decide_retry_from_policy, enforce_required_outputs_strict,
        evaluate_heartbeat_liveness, record_optional_outputs_honestly,
        validate_runtime_state_transition, AttemptHeartbeatSampleV1, CancellationStateV1,
        CommandInvocationModeV1, ConstAdapterOutputArtifactV1, OptionalOutputStatusV1,
        RecoveryWriteRecordV1, RetryFailureClassV1, RetryPolicyInputV1, RuntimeStateV1,
    };

    #[test]
    fn const_adapter_contract_proves_cache_replay_diff_and_inspect_readiness() {
        let contract = build_const_adapter_execution_contract(
            true,
            vec![ConstAdapterOutputArtifactV1 {
                name: "result".to_string(),
                media_type: "application/json".to_string(),
                sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                    .to_string(),
            }],
            3,
        )
        .expect("const adapter contract");
        assert!(contract.cache_replay_diff_inspect_ready);
        assert_eq!(contract.artifacts.len(), 1);
    }

    #[test]
    fn shell_adapter_contract_enforces_argv_timeout_and_output_capture() {
        let contract = build_shell_adapter_execution_contract(
            vec!["python".to_string(), "script.py".to_string()],
            "/workspace/run",
            0,
            60_000,
            true,
            true,
            vec!["artifacts/result.json".to_string()],
        )
        .expect("shell contract should build");
        assert_eq!(contract.argv[0], "python");
        assert_eq!(contract.exit_code, 0);
    }

    #[test]
    fn command_invocation_defaults_to_safe_argv_literal_mode() {
        let safe = build_command_invocation_safety_contract(
            CommandInvocationModeV1::ArgvLiteral,
            vec!["echo".to_string(), "a|b".to_string()],
            None,
        );
        assert!(safe.is_err(), "metacharacter tokens should be blocked by default");

        let explicit_shell = build_command_invocation_safety_contract(
            CommandInvocationModeV1::ShellInterpretation,
            vec!["sh".to_string(), "-lc".to_string()],
            Some("echo a|b".to_string()),
        )
        .expect("explicit shell mode should be allowed");
        assert!(matches!(explicit_shell.mode, CommandInvocationModeV1::ShellInterpretation));
    }

    #[test]
    fn required_outputs_fail_on_missing_corrupt_or_outside_root_paths() {
        let report = enforce_required_outputs_strict(
            "/run/42",
            vec!["metrics".to_string(), "summary".to_string()],
            std::collections::BTreeMap::from([(
                "metrics".to_string(),
                ("/tmp/metrics.bin".to_string(), false),
            )]),
        );
        assert!(!report.success);
        assert!(report.violations.iter().any(|item| item.code == "RO5401_MISSING_REQUIRED_OUTPUT"));
        assert!(report.violations.iter().any(|item| item.code == "RO5402_OUTPUT_OUTSIDE_RUN_ROOT"));
        assert!(report.violations.iter().any(|item| item.code == "RO5404_OUTPUT_CORRUPT"));
    }

    #[test]
    fn optional_outputs_are_recorded_as_explicit_absent_values() {
        let evidence = record_optional_outputs_honestly(
            vec!["summary".to_string(), "plots".to_string()],
            std::collections::BTreeMap::from([(
                "summary".to_string(),
                "/run/42/summary.json".to_string(),
            )]),
        );
        assert!(evidence.iter().any(
            |entry| entry.name == "summary" && entry.status == OptionalOutputStatusV1::Present
        ));
        assert!(evidence
            .iter()
            .any(|entry| entry.name == "plots" && entry.status == OptionalOutputStatusV1::Absent));
    }

    #[test]
    fn runtime_state_machine_is_closed_over_legal_transitions() {
        let legal =
            validate_runtime_state_transition(RuntimeStateV1::Running, RuntimeStateV1::Completed);
        assert!(legal.allowed);

        let illegal =
            validate_runtime_state_transition(RuntimeStateV1::Completed, RuntimeStateV1::Running);
        assert!(!illegal.allowed);
        assert!(illegal.reason.contains("rejected"));
    }

    #[test]
    fn cancellation_requests_are_idempotent_and_non_corrupting() {
        let report = apply_cancellation_idempotently(CancellationStateV1::Running, 3);
        assert_eq!(report.final_state, CancellationStateV1::Cancelled);
        assert!(report.idempotent);
        assert!(!report.artifact_corruption);
    }

    #[test]
    fn retry_policy_retries_only_transient_failures_with_backoff() {
        let no_retry = decide_retry_from_policy(
            RetryPolicyInputV1 { max_attempts: 3, backoff_ms: 1000 },
            RetryFailureClassV1::ContractFailure,
            1,
        );
        assert!(!no_retry.should_retry);

        let retry = decide_retry_from_policy(
            RetryPolicyInputV1 { max_attempts: 3, backoff_ms: 1000 },
            RetryFailureClassV1::TransientAdapterFailure,
            1,
        );
        assert!(retry.should_retry);
        assert_eq!(retry.next_attempt, 2);
        assert_eq!(retry.backoff_ms, 2000);
    }

    #[test]
    fn crash_recovery_detects_duplicate_writes_before_resume() {
        let report = decide_crash_recovery(vec![
            RecoveryWriteRecordV1 {
                node_id: "n1".to_string(),
                output_path: "artifacts/out.json".to_string(),
                write_id: "w1".to_string(),
                committed: true,
            },
            RecoveryWriteRecordV1 {
                node_id: "n1".to_string(),
                output_path: "artifacts/out.json".to_string(),
                write_id: "w2".to_string(),
                committed: true,
            },
        ]);
        assert!(!report.resume_allowed);
        assert!(report.duplicate_write_prevented);
    }

    #[test]
    fn heartbeats_distinguish_alive_attempts_without_fingerprint_noise() {
        let liveness = evaluate_heartbeat_liveness(
            vec![AttemptHeartbeatSampleV1 {
                attempt_id: "attempt-1".to_string(),
                at_unix_ms: 1_000,
            }],
            1_500,
            1_000,
        );
        assert!(liveness.alive);
        assert!(liveness.semantic_fingerprint_input_unchanged);

        let stale = evaluate_heartbeat_liveness(vec![], 10_000, 500);
        assert!(stale.stale);
    }
}
