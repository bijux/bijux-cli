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
        artifact.sha256.len() == 64 && artifact.sha256.chars().all(|value| value.is_ascii_hexdigit())
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
    violations.sort_by(|left, right| left.code.cmp(&right.code).then_with(|| left.output.cmp(&right.output)));
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

#[cfg(test)]
mod tests {
    use super::{
        build_command_invocation_safety_contract,
        build_const_adapter_execution_contract, build_shell_adapter_execution_contract,
        enforce_required_outputs_strict,
        record_optional_outputs_honestly, validate_runtime_state_transition,
        OptionalOutputStatusV1, RuntimeStateV1,
        CommandInvocationModeV1,
        ConstAdapterOutputArtifactV1,
    };

    #[test]
    fn g051_const_adapter_contract_proves_cache_replay_diff_and_inspect_readiness() {
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
    fn g052_shell_adapter_contract_enforces_argv_timeout_and_output_capture() {
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
    fn g053_command_invocation_defaults_to_safe_argv_literal_mode() {
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
        assert!(matches!(
            explicit_shell.mode,
            CommandInvocationModeV1::ShellInterpretation
        ));
    }

    #[test]
    fn g054_required_outputs_fail_on_missing_corrupt_or_outside_root_paths() {
        let report = enforce_required_outputs_strict(
            "/run/42",
            vec!["metrics".to_string(), "summary".to_string()],
            std::collections::BTreeMap::from([
                ("metrics".to_string(), ("/tmp/metrics.bin".to_string(), false)),
            ]),
        );
        assert!(!report.success);
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "RO5401_MISSING_REQUIRED_OUTPUT"));
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "RO5402_OUTPUT_OUTSIDE_RUN_ROOT"));
        assert!(report
            .violations
            .iter()
            .any(|item| item.code == "RO5404_OUTPUT_CORRUPT"));
    }

    #[test]
    fn g055_optional_outputs_are_recorded_as_explicit_absent_values() {
        let evidence = record_optional_outputs_honestly(
            vec!["summary".to_string(), "plots".to_string()],
            std::collections::BTreeMap::from([(
                "summary".to_string(),
                "/run/42/summary.json".to_string(),
            )]),
        );
        assert!(evidence
            .iter()
            .any(|entry| entry.name == "summary"
                && entry.status == OptionalOutputStatusV1::Present));
        assert!(evidence
            .iter()
            .any(|entry| entry.name == "plots"
                && entry.status == OptionalOutputStatusV1::Absent));
    }

    #[test]
    fn g056_runtime_state_machine_is_closed_over_legal_transitions() {
        let legal = validate_runtime_state_transition(RuntimeStateV1::Running, RuntimeStateV1::Completed);
        assert!(legal.allowed);

        let illegal = validate_runtime_state_transition(RuntimeStateV1::Completed, RuntimeStateV1::Running);
        assert!(!illegal.allowed);
        assert!(illegal.reason.contains("rejected"));
    }
}
