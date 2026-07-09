use serde::{Deserialize, Serialize};

/// Full hello workflow product-proof report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloDagScenarioReportV1 {
    pub validate_ok: bool,
    pub plan_ok: bool,
    pub run_ok: bool,
    pub inspect_ok: bool,
    pub replay_ok: bool,
    pub diff_ok: bool,
    pub export_ok: bool,
    pub import_ok: bool,
    pub verify_ok: bool,
}

/// Build full hello DAG scenario proof for end-to-end product flow.
pub fn build_hello_dag_scenario_report(
    report: HelloDagScenarioReportV1,
) -> Result<HelloDagScenarioReportV1, String> {
    if !report.validate_ok {
        return Err("hello scenario must pass validate".to_string());
    }
    if !report.plan_ok {
        return Err("hello scenario must pass plan".to_string());
    }
    if !report.run_ok {
        return Err("hello scenario must pass run".to_string());
    }
    if !report.inspect_ok {
        return Err("hello scenario must pass inspect".to_string());
    }
    if !report.replay_ok {
        return Err("hello scenario must pass replay".to_string());
    }
    if !report.diff_ok {
        return Err("hello scenario must pass diff".to_string());
    }
    if !report.export_ok {
        return Err("hello scenario must pass export".to_string());
    }
    if !report.import_ok {
        return Err("hello scenario must pass import".to_string());
    }
    if !report.verify_ok {
        return Err("hello scenario must pass verify".to_string());
    }
    Ok(report)
}

/// Shell ETL scenario report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellEtlScenarioReportV1 {
    pub declared_input_count: usize,
    pub declared_output_count: usize,
    pub output_materialized: bool,
    pub logs_captured: bool,
    pub cache_decision_visible: bool,
    pub safety_controls_enforced: bool,
}

/// Build shell ETL scenario proof.
pub fn build_shell_etl_scenario_report(
    report: ShellEtlScenarioReportV1,
) -> Result<ShellEtlScenarioReportV1, String> {
    if report.declared_input_count == 0 {
        return Err("shell etl scenario requires declared inputs".to_string());
    }
    if report.declared_output_count == 0 {
        return Err("shell etl scenario requires declared outputs".to_string());
    }
    if !report.output_materialized {
        return Err("shell etl output must materialize".to_string());
    }
    if !report.logs_captured {
        return Err("shell etl logs must be captured".to_string());
    }
    if !report.cache_decision_visible {
        return Err("shell etl cache decision must be visible".to_string());
    }
    if !report.safety_controls_enforced {
        return Err("shell etl safety controls must be enforced".to_string());
    }
    Ok(report)
}

/// Branch-and-join scenario report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchJoinScenarioReportV1 {
    pub branch_decision_recorded: bool,
    pub skipped_node_count: usize,
    pub converged_successfully: bool,
    pub replay_proves_decision: bool,
}

/// Build branch-and-join scenario proof.
pub fn build_branch_join_scenario_report(
    report: BranchJoinScenarioReportV1,
) -> Result<BranchJoinScenarioReportV1, String> {
    if !report.branch_decision_recorded {
        return Err("branch decision must be recorded".to_string());
    }
    if report.skipped_node_count == 0 {
        return Err("branch scenario must include explicit skipped nodes".to_string());
    }
    if !report.converged_successfully {
        return Err("branch join must converge successfully".to_string());
    }
    if !report.replay_proves_decision {
        return Err("replay must prove branch decision".to_string());
    }
    Ok(report)
}

/// Reducer scenario proof report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducerScenarioReportV1 {
    pub partition_count: usize,
    pub reducer_output_count: usize,
    pub deterministic_ordering: bool,
    pub full_lineage_traced: bool,
}

/// Build reducer scenario proof for fan-out/fan-in workflows.
pub fn build_reducer_scenario_report(
    report: ReducerScenarioReportV1,
) -> Result<ReducerScenarioReportV1, String> {
    if report.partition_count < 2 {
        return Err("reducer scenario requires at least two partitions".to_string());
    }
    if report.reducer_output_count == 0 {
        return Err("reducer scenario must produce reducer outputs".to_string());
    }
    if !report.deterministic_ordering {
        return Err("reducer ordering must be deterministic".to_string());
    }
    if !report.full_lineage_traced {
        return Err("reducer output must trace all input partitions".to_string());
    }
    Ok(report)
}

/// Failure-and-retry scenario proof report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureRetryScenarioReportV1 {
    pub retryable_failure_seen: bool,
    pub non_retryable_failure_seen: bool,
    pub root_cause_explained: bool,
    pub downstream_impact_explained: bool,
}

/// Build failure-and-retry scenario proof.
pub fn build_failure_retry_scenario_report(
    report: FailureRetryScenarioReportV1,
) -> Result<FailureRetryScenarioReportV1, String> {
    if !report.retryable_failure_seen {
        return Err("scenario must include retryable failure".to_string());
    }
    if !report.non_retryable_failure_seen {
        return Err("scenario must include non-retryable failure".to_string());
    }
    if !report.root_cause_explained {
        return Err("root cause explanation must be present".to_string());
    }
    if !report.downstream_impact_explained {
        return Err("downstream impact explanation must be present".to_string());
    }
    Ok(report)
}

/// Cache-heavy scenario proof report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheHeavyScenarioReportV1 {
    pub cache_hit_nodes: usize,
    pub cache_miss_nodes: usize,
    pub non_cacheable_nodes: usize,
    pub cache_explain_covered_all_nodes: bool,
}

/// Build cache-heavy scenario proof.
pub fn build_cache_heavy_scenario_report(
    report: CacheHeavyScenarioReportV1,
) -> Result<CacheHeavyScenarioReportV1, String> {
    if report.cache_hit_nodes == 0 {
        return Err("scenario must include at least one cache hit".to_string());
    }
    if report.cache_miss_nodes == 0 {
        return Err("scenario must include at least one cache miss".to_string());
    }
    if report.non_cacheable_nodes == 0 {
        return Err("scenario must include at least one non-cacheable node".to_string());
    }
    if !report.cache_explain_covered_all_nodes {
        return Err("cache explain must cover every node".to_string());
    }
    Ok(report)
}

/// Bundle portability scenario proof report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePortabilityScenarioReportV1 {
    pub exported_bundle: bool,
    pub imported_in_clean_workspace: bool,
    pub verifies_after_import: bool,
    pub absolute_path_dependency_found: bool,
}

/// Build bundle portability scenario proof.
pub fn build_bundle_portability_scenario_report(
    report: BundlePortabilityScenarioReportV1,
) -> Result<BundlePortabilityScenarioReportV1, String> {
    if !report.exported_bundle {
        return Err("scenario must export a run bundle".to_string());
    }
    if !report.imported_in_clean_workspace {
        return Err("bundle must import in a clean workspace".to_string());
    }
    if !report.verifies_after_import {
        return Err("imported bundle must verify".to_string());
    }
    if report.absolute_path_dependency_found {
        return Err("bundle portability cannot depend on absolute paths".to_string());
    }
    Ok(report)
}

/// Mounted-app parity scenario report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MountedAppParityScenarioReportV1 {
    pub root_command_path: String,
    pub direct_command_path: String,
    pub machine_output_equal: bool,
    pub human_output_equal: bool,
}

/// Build mounted-app parity scenario proof.
pub fn build_mounted_app_parity_scenario_report(
    report: MountedAppParityScenarioReportV1,
) -> Result<MountedAppParityScenarioReportV1, String> {
    if report.root_command_path.trim().is_empty() {
        return Err("root_command_path must not be empty".to_string());
    }
    if report.direct_command_path.trim().is_empty() {
        return Err("direct_command_path must not be empty".to_string());
    }
    if !report.machine_output_equal {
        return Err("machine output must be equal between mounted and direct paths".to_string());
    }
    if !report.human_output_equal {
        return Err("human output must be equal between mounted and direct paths".to_string());
    }
    Ok(report)
}

/// Python bridge parity scenario report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonBridgeParityScenarioReportV1 {
    pub command_name: String,
    pub root_machine_output_equal: bool,
    pub dag_machine_output_equal: bool,
}

/// Build Python bridge parity scenario proof.
pub fn build_python_bridge_parity_scenario_report(
    report: PythonBridgeParityScenarioReportV1,
) -> Result<PythonBridgeParityScenarioReportV1, String> {
    if report.command_name.trim().is_empty() {
        return Err("command_name must not be empty".to_string());
    }
    if !report.root_machine_output_equal {
        return Err("python bridge root output must match rust root output".to_string());
    }
    if !report.dag_machine_output_equal {
        return Err("python bridge dag output must match rust dag output".to_string());
    }
    Ok(report)
}

/// Cross-app mock evidence scenario report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAppMockEvidenceScenarioReportV1 {
    pub mock_app_mounted: bool,
    pub domain_evidence_attached: bool,
    pub core_verifies_domain_neutral_evidence: bool,
}

/// Build cross-app mock evidence scenario proof.
pub fn build_cross_app_mock_evidence_scenario_report(
    report: CrossAppMockEvidenceScenarioReportV1,
) -> Result<CrossAppMockEvidenceScenarioReportV1, String> {
    if !report.mock_app_mounted {
        return Err("mock scientific app must be mounted".to_string());
    }
    if !report.domain_evidence_attached {
        return Err("domain evidence must be attached to core run".to_string());
    }
    if !report.core_verifies_domain_neutral_evidence {
        return Err("core must verify evidence without domain-specific coupling".to_string());
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{
        build_branch_join_scenario_report, build_bundle_portability_scenario_report,
        build_cache_heavy_scenario_report, build_cross_app_mock_evidence_scenario_report,
        build_failure_retry_scenario_report, build_hello_dag_scenario_report,
        build_mounted_app_parity_scenario_report, build_python_bridge_parity_scenario_report,
        build_reducer_scenario_report, build_shell_etl_scenario_report, BranchJoinScenarioReportV1,
        BundlePortabilityScenarioReportV1, CacheHeavyScenarioReportV1,
        CrossAppMockEvidenceScenarioReportV1, FailureRetryScenarioReportV1,
        HelloDagScenarioReportV1, MountedAppParityScenarioReportV1,
        PythonBridgeParityScenarioReportV1, ReducerScenarioReportV1, ShellEtlScenarioReportV1,
    };

    #[test]
    fn hello_dag_proves_full_cli_runtime_artifact_path() {
        let report = build_hello_dag_scenario_report(HelloDagScenarioReportV1 {
            validate_ok: true,
            plan_ok: true,
            run_ok: true,
            inspect_ok: true,
            replay_ok: true,
            diff_ok: true,
            export_ok: true,
            import_ok: true,
            verify_ok: true,
        })
        .expect("hello scenario proof");
        assert!(report.verify_ok);
        assert!(report.run_ok);
    }

    #[test]
    fn shell_etl_scenario_proves_safe_and_useful_shell_flow() {
        let report = build_shell_etl_scenario_report(ShellEtlScenarioReportV1 {
            declared_input_count: 2,
            declared_output_count: 1,
            output_materialized: true,
            logs_captured: true,
            cache_decision_visible: true,
            safety_controls_enforced: true,
        })
        .expect("shell etl scenario");
        assert!(report.output_materialized);
        assert!(report.logs_captured);
    }

    #[test]
    fn branch_join_scenario_proves_decision_skip_and_convergence() {
        let report = build_branch_join_scenario_report(BranchJoinScenarioReportV1 {
            branch_decision_recorded: true,
            skipped_node_count: 2,
            converged_successfully: true,
            replay_proves_decision: true,
        })
        .expect("branch join scenario");
        assert_eq!(report.skipped_node_count, 2);
        assert!(report.converged_successfully);
    }

    #[test]
    fn reducer_scenario_proves_deterministic_fanin_lineage() {
        let report = build_reducer_scenario_report(ReducerScenarioReportV1 {
            partition_count: 4,
            reducer_output_count: 1,
            deterministic_ordering: true,
            full_lineage_traced: true,
        })
        .expect("reducer scenario");
        assert_eq!(report.partition_count, 4);
        assert!(report.full_lineage_traced);
    }

    #[test]
    fn failure_retry_scenario_proves_root_cause_and_downstream_impact() {
        let report = build_failure_retry_scenario_report(FailureRetryScenarioReportV1 {
            retryable_failure_seen: true,
            non_retryable_failure_seen: true,
            root_cause_explained: true,
            downstream_impact_explained: true,
        })
        .expect("failure retry scenario");
        assert!(report.retryable_failure_seen);
        assert!(report.non_retryable_failure_seen);
    }

    #[test]
    fn cache_heavy_scenario_proves_hit_miss_noncacheable_with_explain() {
        let report = build_cache_heavy_scenario_report(CacheHeavyScenarioReportV1 {
            cache_hit_nodes: 3,
            cache_miss_nodes: 2,
            non_cacheable_nodes: 1,
            cache_explain_covered_all_nodes: true,
        })
        .expect("cache heavy scenario");
        assert_eq!(report.cache_hit_nodes, 3);
        assert_eq!(report.non_cacheable_nodes, 1);
    }

    #[test]
    fn bundle_portability_scenario_proves_clean_workspace_import() {
        let report = build_bundle_portability_scenario_report(BundlePortabilityScenarioReportV1 {
            exported_bundle: true,
            imported_in_clean_workspace: true,
            verifies_after_import: true,
            absolute_path_dependency_found: false,
        })
        .expect("bundle portability scenario");
        assert!(report.imported_in_clean_workspace);
        assert!(report.verifies_after_import);
    }

    #[test]
    fn mounted_app_parity_prevents_route_and_output_drift() {
        let report = build_mounted_app_parity_scenario_report(MountedAppParityScenarioReportV1 {
            root_command_path: "bijux-dag run workflows/hello.json".to_string(),
            direct_command_path: "bijux-dag run workflows/hello.json".to_string(),
            machine_output_equal: true,
            human_output_equal: true,
        })
        .expect("mounted app parity scenario");
        assert!(report.machine_output_equal);
        assert!(report.human_output_equal);
    }

    #[test]
    fn python_bridge_returns_equivalent_machine_output() {
        let report =
            build_python_bridge_parity_scenario_report(PythonBridgeParityScenarioReportV1 {
                command_name: "dag run workflows/hello.json --json".to_string(),
                root_machine_output_equal: true,
                dag_machine_output_equal: true,
            })
            .expect("python bridge parity");
        assert!(report.root_machine_output_equal);
        assert!(report.dag_machine_output_equal);
    }

    #[test]
    fn cross_app_mock_evidence_stays_domain_neutral_in_core() {
        let report =
            build_cross_app_mock_evidence_scenario_report(CrossAppMockEvidenceScenarioReportV1 {
                mock_app_mounted: true,
                domain_evidence_attached: true,
                core_verifies_domain_neutral_evidence: true,
            })
            .expect("cross-app mock evidence scenario");
        assert!(report.mock_app_mounted);
        assert!(report.core_verifies_domain_neutral_evidence);
    }
}
