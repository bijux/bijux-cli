use crate::commands::{DagCli, PerformanceCommands};
use crate::routes::simulation_io::load_json_file;
use crate::{emit_json, ExitCode};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct LatencyBudgetSimulation {
    p50_budget_ms: u64,
    p95_budget_ms: u64,
    measurements: Vec<LatencyMeasurement>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct LatencyMeasurement {
    name: String,
    p50_ms: u64,
    p95_ms: u64,
}

#[derive(Debug, Serialize)]
struct LatencyBudgetReport {
    policy_lane: &'static str,
    p50_budget_ms: u64,
    p95_budget_ms: u64,
    within_budget: bool,
    breached_measurements: Vec<String>,
    measurements: Vec<LatencyMeasurement>,
}

#[derive(Debug, serde::Deserialize)]
struct LargeGraphCorpusSimulation {
    target_node_counts: Vec<usize>,
    avg_fanout: usize,
    expansion_multiplier: usize,
    max_generated_nodes: usize,
}

#[derive(Debug, Serialize)]
struct LargeGraphCorpusEntry {
    target_nodes: usize,
    generated_nodes: usize,
    generated_edges: usize,
    expanded_nodes: usize,
}

#[derive(Debug, Serialize)]
struct LargeGraphCorpusReport {
    policy_lane: &'static str,
    corpus_entries: Vec<LargeGraphCorpusEntry>,
    includes_100_nodes: bool,
    includes_1k_nodes: bool,
    includes_10k_nodes: bool,
    within_generation_bound: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CanonicalizationProfileSimulation {
    samples: Vec<CanonicalizationSample>,
    max_allowed_regression_pct: f64,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct CanonicalizationSample {
    fixture: String,
    before_ms: u64,
    after_ms: u64,
}

#[derive(Debug, Serialize)]
struct CanonicalizationProfileReport {
    policy_lane: &'static str,
    average_before_ms: f64,
    average_after_ms: f64,
    average_speedup_pct: f64,
    regression_fixtures: Vec<String>,
    within_regression_budget: bool,
    samples: Vec<CanonicalizationSample>,
}

#[derive(Debug, serde::Deserialize)]
struct SchedulerChurnSimulation {
    ready_queue_ops: u64,
    trigger_evaluations: u64,
    branch_propagations: u64,
    retry_events: u64,
    churn_budget_ops: u64,
}

#[derive(Debug, Serialize)]
struct SchedulerChurnReport {
    policy_lane: &'static str,
    churn_ops_total: u64,
    churn_budget_ops: u64,
    within_budget: bool,
    retry_storm_detected: bool,
    pressure_index: f64,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ArtifactWriteProfileSimulation {
    small_write_count: u64,
    large_write_count: u64,
    nested_dir_count: u64,
    bytes_written: u64,
    elapsed_ms: u64,
    max_elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
struct ArtifactWriteProfileReport {
    policy_lane: &'static str,
    write_ops_total: u64,
    bytes_written: u64,
    elapsed_ms: u64,
    throughput_bytes_per_sec: f64,
    within_elapsed_budget: bool,
    nested_layout_present: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MemoryCeilingsSimulation {
    phase_metrics_mb: std::collections::BTreeMap<String, u64>,
    phase_ceilings_mb: std::collections::BTreeMap<String, u64>,
}

#[derive(Debug, Serialize)]
struct MemoryCeilingsReport {
    policy_lane: &'static str,
    phase_metrics_mb: std::collections::BTreeMap<String, u64>,
    phase_ceilings_mb: std::collections::BTreeMap<String, u64>,
    over_ceiling_phases: Vec<String>,
    within_ceiling: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamingOutputSimulation {
    stream_name: String,
    total_bytes: u64,
    chunk_bytes: u64,
    peak_in_memory_bytes: u64,
    max_in_memory_bytes: u64,
}

#[derive(Debug, Serialize)]
struct StreamingOutputReport {
    policy_lane: &'static str,
    stream_name: String,
    total_bytes: u64,
    chunk_bytes: u64,
    peak_in_memory_bytes: u64,
    max_in_memory_bytes: u64,
    bounded_memory: bool,
    streaming_effective: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RunHistoryCompactionSimulation {
    records_before: u64,
    records_after: u64,
    bytes_before: u64,
    bytes_after: u64,
    query_p95_before_ms: u64,
    query_p95_after_ms: u64,
    max_query_p95_ms: u64,
}

#[derive(Debug, Serialize)]
struct RunHistoryCompactionReport {
    policy_lane: &'static str,
    records_before: u64,
    records_after: u64,
    bytes_before: u64,
    bytes_after: u64,
    compaction_ratio: f64,
    query_p95_before_ms: u64,
    query_p95_after_ms: u64,
    query_within_budget: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct BenchmarkReportGovernanceSimulation {
    fixture_id: String,
    hardware_note: String,
    run_version: String,
    variance_pct: f64,
    max_variance_pct: f64,
    reproducible: bool,
}

#[derive(Debug, Serialize)]
struct BenchmarkReportGovernanceReport {
    policy_lane: &'static str,
    fixture_id: String,
    hardware_note: String,
    run_version: String,
    variance_pct: f64,
    max_variance_pct: f64,
    reproducible: bool,
    governance_passed: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PerformanceRegressionGatesSimulation {
    metrics: Vec<RegressionMetricSample>,
    max_regression_pct: f64,
    override_applied: bool,
    override_reason: Option<String>,
    override_ticket: Option<String>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct RegressionMetricSample {
    metric: String,
    baseline_ms: u64,
    current_ms: u64,
}

#[derive(Debug, Serialize)]
struct PerformanceRegressionGatesReport {
    policy_lane: &'static str,
    failing_metrics: Vec<String>,
    gates_passed: bool,
    override_applied: bool,
    override_valid: bool,
    override_reason: Option<String>,
    override_ticket: Option<String>,
    gaps: Vec<String>,
}

fn latency_budgets_payload(simulation: &Path) -> Result<LatencyBudgetReport, ExitCode> {
    let simulation: LatencyBudgetSimulation = load_json_file(simulation)?;
    let mut breached_measurements = simulation
        .measurements
        .iter()
        .filter(|measurement| {
            measurement.p50_ms > simulation.p50_budget_ms
                || measurement.p95_ms > simulation.p95_budget_ms
        })
        .map(|measurement| measurement.name.clone())
        .collect::<Vec<_>>();
    breached_measurements.sort();
    let within_budget = breached_measurements.is_empty();
    Ok(LatencyBudgetReport {
        policy_lane: "ENFORCED",
        p50_budget_ms: simulation.p50_budget_ms,
        p95_budget_ms: simulation.p95_budget_ms,
        within_budget,
        breached_measurements,
        measurements: simulation.measurements,
    })
}

fn large_graph_corpus_payload(simulation: &Path) -> Result<LargeGraphCorpusReport, ExitCode> {
    let simulation: LargeGraphCorpusSimulation = load_json_file(simulation)?;
    let mut corpus_entries = simulation
        .target_node_counts
        .iter()
        .map(|target| {
            let generated_nodes = *target;
            let generated_edges = generated_nodes.saturating_mul(simulation.avg_fanout);
            let expanded_nodes = generated_nodes.saturating_mul(simulation.expansion_multiplier);
            LargeGraphCorpusEntry { target_nodes: *target, generated_nodes, generated_edges, expanded_nodes }
        })
        .collect::<Vec<_>>();
    corpus_entries.sort_by_key(|entry| entry.target_nodes);

    let includes_100_nodes = corpus_entries.iter().any(|entry| entry.target_nodes == 100);
    let includes_1k_nodes = corpus_entries.iter().any(|entry| entry.target_nodes == 1_000);
    let includes_10k_nodes = corpus_entries.iter().any(|entry| entry.target_nodes == 10_000);
    let within_generation_bound = corpus_entries
        .iter()
        .all(|entry| entry.expanded_nodes <= simulation.max_generated_nodes);
    let mut gaps = Vec::new();
    if !includes_100_nodes {
        gaps.push("corpus does not include 100-node fixture".to_string());
    }
    if !includes_1k_nodes {
        gaps.push("corpus does not include 1k-node fixture".to_string());
    }
    if !includes_10k_nodes {
        gaps.push("corpus does not include 10k-node fixture".to_string());
    }
    if !within_generation_bound {
        gaps.push("expanded corpus exceeds configured generation bound".to_string());
    }

    Ok(LargeGraphCorpusReport {
        policy_lane: "ENFORCED",
        corpus_entries,
        includes_100_nodes,
        includes_1k_nodes,
        includes_10k_nodes,
        within_generation_bound,
        gaps,
    })
}

fn canonicalization_profile_payload(
    simulation: &Path,
) -> Result<CanonicalizationProfileReport, ExitCode> {
    let simulation: CanonicalizationProfileSimulation = load_json_file(simulation)?;
    let sample_count = simulation.samples.len().max(1) as f64;
    let total_before = simulation.samples.iter().map(|sample| sample.before_ms as f64).sum::<f64>();
    let total_after = simulation.samples.iter().map(|sample| sample.after_ms as f64).sum::<f64>();
    let average_before_ms = total_before / sample_count;
    let average_after_ms = total_after / sample_count;
    let average_speedup_pct = if average_before_ms == 0.0 {
        0.0
    } else {
        ((average_before_ms - average_after_ms) / average_before_ms) * 100.0
    };
    let mut regression_fixtures = simulation
        .samples
        .iter()
        .filter(|sample| {
            let regression_pct = if sample.before_ms == 0 {
                0.0
            } else {
                ((sample.after_ms as f64 - sample.before_ms as f64) / sample.before_ms as f64) * 100.0
            };
            regression_pct > simulation.max_allowed_regression_pct
        })
        .map(|sample| sample.fixture.clone())
        .collect::<Vec<_>>();
    regression_fixtures.sort();
    let within_regression_budget = regression_fixtures.is_empty();
    Ok(CanonicalizationProfileReport {
        policy_lane: "ENFORCED",
        average_before_ms,
        average_after_ms,
        average_speedup_pct,
        regression_fixtures,
        within_regression_budget,
        samples: simulation.samples,
    })
}

fn scheduler_churn_payload(simulation: &Path) -> Result<SchedulerChurnReport, ExitCode> {
    let simulation: SchedulerChurnSimulation = load_json_file(simulation)?;
    let churn_ops_total = simulation
        .ready_queue_ops
        .saturating_add(simulation.trigger_evaluations)
        .saturating_add(simulation.branch_propagations)
        .saturating_add(simulation.retry_events);
    let within_budget = churn_ops_total <= simulation.churn_budget_ops;
    let retry_storm_detected = simulation.retry_events > simulation.ready_queue_ops.max(1) / 2;
    let pressure_index = if simulation.churn_budget_ops == 0 {
        0.0
    } else {
        churn_ops_total as f64 / simulation.churn_budget_ops as f64
    };
    let mut gaps = Vec::new();
    if !within_budget {
        gaps.push("scheduler churn exceeds configured budget".to_string());
    }
    if retry_storm_detected {
        gaps.push("retry storm signal detected in scheduler workload".to_string());
    }
    Ok(SchedulerChurnReport {
        policy_lane: "ENFORCED",
        churn_ops_total,
        churn_budget_ops: simulation.churn_budget_ops,
        within_budget,
        retry_storm_detected,
        pressure_index,
        gaps,
    })
}

fn artifact_write_profile_payload(
    simulation: &Path,
) -> Result<ArtifactWriteProfileReport, ExitCode> {
    let simulation: ArtifactWriteProfileSimulation = load_json_file(simulation)?;
    let write_ops_total = simulation
        .small_write_count
        .saturating_add(simulation.large_write_count);
    let throughput_bytes_per_sec = if simulation.elapsed_ms == 0 {
        0.0
    } else {
        simulation.bytes_written as f64 / (simulation.elapsed_ms as f64 / 1000.0)
    };
    let within_elapsed_budget = simulation.elapsed_ms <= simulation.max_elapsed_ms;
    let nested_layout_present = simulation.nested_dir_count > 0;
    let mut gaps = Vec::new();
    if !within_elapsed_budget {
        gaps.push("artifact write elapsed time exceeds budget".to_string());
    }
    if !nested_layout_present {
        gaps.push("artifact write profile lacks nested directory coverage".to_string());
    }
    if write_ops_total == 0 {
        gaps.push("artifact write profile has no write operations".to_string());
    }
    Ok(ArtifactWriteProfileReport {
        policy_lane: "ENFORCED",
        write_ops_total,
        bytes_written: simulation.bytes_written,
        elapsed_ms: simulation.elapsed_ms,
        throughput_bytes_per_sec,
        within_elapsed_budget,
        nested_layout_present,
        gaps,
    })
}

fn memory_ceilings_payload(simulation: &Path) -> Result<MemoryCeilingsReport, ExitCode> {
    let simulation: MemoryCeilingsSimulation = load_json_file(simulation)?;
    let mut over_ceiling_phases = Vec::new();
    let mut gaps = Vec::new();
    for (phase, observed_mb) in &simulation.phase_metrics_mb {
        match simulation.phase_ceilings_mb.get(phase) {
            Some(ceiling_mb) => {
                if observed_mb > ceiling_mb {
                    over_ceiling_phases.push(phase.clone());
                    gaps.push(format!("{phase} exceeds memory ceiling"));
                }
            }
            None => {
                gaps.push(format!("{phase} has no configured memory ceiling"));
            }
        }
    }
    over_ceiling_phases.sort();
    let within_ceiling = over_ceiling_phases.is_empty();
    Ok(MemoryCeilingsReport {
        policy_lane: "ENFORCED",
        phase_metrics_mb: simulation.phase_metrics_mb,
        phase_ceilings_mb: simulation.phase_ceilings_mb,
        over_ceiling_phases,
        within_ceiling,
        gaps,
    })
}

fn streaming_output_payload(simulation: &Path) -> Result<StreamingOutputReport, ExitCode> {
    let simulation: StreamingOutputSimulation = load_json_file(simulation)?;
    let bounded_memory = simulation.peak_in_memory_bytes <= simulation.max_in_memory_bytes;
    let streaming_effective = simulation.chunk_bytes > 0
        && simulation.chunk_bytes < simulation.total_bytes
        && bounded_memory;
    let mut gaps = Vec::new();
    if !bounded_memory {
        gaps.push("stream handling exceeded memory ceiling".to_string());
    }
    if simulation.chunk_bytes == 0 {
        gaps.push("chunk size must be greater than zero".to_string());
    }
    if simulation.total_bytes > 0 && simulation.chunk_bytes >= simulation.total_bytes {
        gaps.push("chunk size indicates non-streaming full-buffer behavior".to_string());
    }
    Ok(StreamingOutputReport {
        policy_lane: "ENFORCED",
        stream_name: simulation.stream_name,
        total_bytes: simulation.total_bytes,
        chunk_bytes: simulation.chunk_bytes,
        peak_in_memory_bytes: simulation.peak_in_memory_bytes,
        max_in_memory_bytes: simulation.max_in_memory_bytes,
        bounded_memory,
        streaming_effective,
        gaps,
    })
}

fn run_history_compaction_payload(
    simulation: &Path,
) -> Result<RunHistoryCompactionReport, ExitCode> {
    let simulation: RunHistoryCompactionSimulation = load_json_file(simulation)?;
    let compaction_ratio = if simulation.bytes_before == 0 {
        1.0
    } else {
        simulation.bytes_after as f64 / simulation.bytes_before as f64
    };
    let query_within_budget = simulation.query_p95_after_ms <= simulation.max_query_p95_ms;
    let mut gaps = Vec::new();
    if simulation.records_after > simulation.records_before {
        gaps.push("compaction increased run-history record count".to_string());
    }
    if simulation.bytes_after > simulation.bytes_before {
        gaps.push("compaction increased run-history storage bytes".to_string());
    }
    if !query_within_budget {
        gaps.push("post-compaction run-history query p95 exceeds budget".to_string());
    }
    Ok(RunHistoryCompactionReport {
        policy_lane: "ENFORCED",
        records_before: simulation.records_before,
        records_after: simulation.records_after,
        bytes_before: simulation.bytes_before,
        bytes_after: simulation.bytes_after,
        compaction_ratio,
        query_p95_before_ms: simulation.query_p95_before_ms,
        query_p95_after_ms: simulation.query_p95_after_ms,
        query_within_budget,
        gaps,
    })
}

fn benchmark_report_governance_payload(
    simulation: &Path,
) -> Result<BenchmarkReportGovernanceReport, ExitCode> {
    let simulation: BenchmarkReportGovernanceSimulation = load_json_file(simulation)?;
    let mut gaps = Vec::new();
    if simulation.fixture_id.trim().is_empty() {
        gaps.push("benchmark report fixture identity is missing".to_string());
    }
    if simulation.hardware_note.trim().is_empty() {
        gaps.push("benchmark report hardware note is missing".to_string());
    }
    if simulation.run_version.trim().is_empty() {
        gaps.push("benchmark report version is missing".to_string());
    }
    if simulation.variance_pct > simulation.max_variance_pct {
        gaps.push("benchmark report variance exceeds allowed budget".to_string());
    }
    if !simulation.reproducible {
        gaps.push("benchmark run is not reproducible".to_string());
    }
    let governance_passed = gaps.is_empty();
    Ok(BenchmarkReportGovernanceReport {
        policy_lane: "ENFORCED",
        fixture_id: simulation.fixture_id,
        hardware_note: simulation.hardware_note,
        run_version: simulation.run_version,
        variance_pct: simulation.variance_pct,
        max_variance_pct: simulation.max_variance_pct,
        reproducible: simulation.reproducible,
        governance_passed,
        gaps,
    })
}

fn performance_regression_gates_payload(
    simulation: &Path,
) -> Result<PerformanceRegressionGatesReport, ExitCode> {
    let simulation: PerformanceRegressionGatesSimulation = load_json_file(simulation)?;
    let mut failing_metrics = simulation
        .metrics
        .iter()
        .filter(|sample| {
            if sample.baseline_ms == 0 {
                false
            } else {
                ((sample.current_ms as f64 - sample.baseline_ms as f64) / sample.baseline_ms as f64) * 100.0
                    > simulation.max_regression_pct
            }
        })
        .map(|sample| sample.metric.clone())
        .collect::<Vec<_>>();
    failing_metrics.sort();

    let override_valid = if simulation.override_applied {
        simulation
            .override_reason
            .as_ref()
            .is_some_and(|reason| !reason.trim().is_empty())
            && simulation
                .override_ticket
                .as_ref()
                .is_some_and(|ticket| !ticket.trim().is_empty())
    } else {
        true
    };
    let gates_passed = failing_metrics.is_empty() || (simulation.override_applied && override_valid);
    let mut gaps = Vec::new();
    if !failing_metrics.is_empty() && !simulation.override_applied {
        gaps.push("performance regression gate failure requires override or fix".to_string());
    }
    if simulation.override_applied && !override_valid {
        gaps.push("performance regression override is missing reason or ticket".to_string());
    }
    Ok(PerformanceRegressionGatesReport {
        policy_lane: "ENFORCED",
        failing_metrics,
        gates_passed,
        override_applied: simulation.override_applied,
        override_valid,
        override_reason: simulation.override_reason,
        override_ticket: simulation.override_ticket,
        gaps,
    })
}

pub(crate) fn handle_performance_command(
    cli: &DagCli,
    command: &PerformanceCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        PerformanceCommands::LatencyBudgets { simulation } => {
            let payload =
                serde_json::to_value(latency_budgets_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.latency-budgets",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::LargeGraphCorpus { simulation } => {
            let payload = serde_json::to_value(large_graph_corpus_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.large-graph-corpus",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::CanonicalizationProfile { simulation } => {
            let payload = serde_json::to_value(canonicalization_profile_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.canonicalization-profile",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::SchedulerChurn { simulation } => {
            let payload =
                serde_json::to_value(scheduler_churn_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.scheduler-churn",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::ArtifactWriteProfile { simulation } => {
            let payload = serde_json::to_value(artifact_write_profile_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.artifact-write-profile",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::MemoryCeilings { simulation } => {
            let payload =
                serde_json::to_value(memory_ceilings_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.memory-ceilings",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::StreamingOutput { simulation } => {
            let payload =
                serde_json::to_value(streaming_output_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.streaming-output",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::RunHistoryCompaction { simulation } => {
            let payload =
                serde_json::to_value(run_history_compaction_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.run-history-compaction",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::BenchmarkReportGovernance { simulation } => {
            let payload = serde_json::to_value(benchmark_report_governance_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.benchmark-report-governance",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        PerformanceCommands::PerformanceRegressionGates { simulation } => {
            let payload = serde_json::to_value(performance_regression_gates_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.performance-regression-gates",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_performance_command;
    use crate::commands::{Commands, DagCli, PerformanceCommands};
    use crate::ExitCode;

    fn quiet_json_cli(command: PerformanceCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Performance { command } }
    }

    #[test]
    fn performance_latency_budgets_accepts_within_budget_measurements() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("latency-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "p50_budget_ms":120,
              "p95_budget_ms":300,
              "measurements":[
                {"name":"route_dispatch","p50_ms":20,"p95_ms":90},
                {"name":"graph_parse","p50_ms":40,"p95_ms":150}
              ]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(PerformanceCommands::LatencyBudgets { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::LatencyBudgets { simulation: simulation.clone() },
        )
        .expect("latency budgets");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::latency_budgets_payload(&simulation).expect("report");
        assert!(report.within_budget);
        assert!(report.breached_measurements.is_empty());
        assert_eq!(report.policy_lane, "ENFORCED");
    }

    #[test]
    fn performance_latency_budgets_flags_p50_and_p95_regressions() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("latency-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "p50_budget_ms":120,
              "p95_budget_ms":300,
              "measurements":[
                {"name":"route_dispatch","p50_ms":121,"p95_ms":290},
                {"name":"graph_parse","p50_ms":80,"p95_ms":301}
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::latency_budgets_payload(&simulation).expect("report");
        assert!(!report.within_budget);
        assert_eq!(
            report.breached_measurements,
            vec!["graph_parse".to_string(), "route_dispatch".to_string()]
        );
    }

    #[test]
    fn performance_large_graph_corpus_accepts_required_scale_targets() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("corpus-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "target_node_counts":[100,1000,10000],
              "avg_fanout":2,
              "expansion_multiplier":2,
              "max_generated_nodes":25000
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(PerformanceCommands::LargeGraphCorpus { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::LargeGraphCorpus { simulation: simulation.clone() },
        )
        .expect("large graph corpus");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::large_graph_corpus_payload(&simulation).expect("report");
        assert!(report.includes_100_nodes);
        assert!(report.includes_1k_nodes);
        assert!(report.includes_10k_nodes);
        assert!(report.within_generation_bound);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_large_graph_corpus_flags_missing_targets_and_bounds() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("corpus-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "target_node_counts":[128,2048],
              "avg_fanout":3,
              "expansion_multiplier":8,
              "max_generated_nodes":10000
            }"#,
        )
        .expect("write simulation");
        let report = super::large_graph_corpus_payload(&simulation).expect("report");
        assert!(!report.includes_100_nodes);
        assert!(!report.includes_1k_nodes);
        assert!(!report.includes_10k_nodes);
        assert!(!report.within_generation_bound);
        for expected in [
            "corpus does not include 100-node fixture",
            "corpus does not include 1k-node fixture",
            "corpus does not include 10k-node fixture",
            "expanded corpus exceeds configured generation bound",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn performance_canonicalization_profile_accepts_speedup() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("canonicalization-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "samples":[
                {"fixture":"small","before_ms":40,"after_ms":30},
                {"fixture":"medium","before_ms":80,"after_ms":60},
                {"fixture":"large","before_ms":120,"after_ms":90}
              ],
              "max_allowed_regression_pct":5.0
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(PerformanceCommands::CanonicalizationProfile { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::CanonicalizationProfile { simulation: simulation.clone() },
        )
        .expect("canonicalization profile");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::canonicalization_profile_payload(&simulation).expect("report");
        assert!(report.within_regression_budget);
        assert!(report.average_speedup_pct > 0.0);
        assert!(report.regression_fixtures.is_empty());
    }

    #[test]
    fn performance_canonicalization_profile_flags_over_budget_regression() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("canonicalization-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "samples":[
                {"fixture":"small","before_ms":40,"after_ms":60},
                {"fixture":"large","before_ms":120,"after_ms":121}
              ],
              "max_allowed_regression_pct":10.0
            }"#,
        )
        .expect("write simulation");
        let report = super::canonicalization_profile_payload(&simulation).expect("report");
        assert!(!report.within_regression_budget);
        assert_eq!(report.regression_fixtures, vec!["small".to_string()]);
    }

    #[test]
    fn performance_scheduler_churn_accepts_budgeted_workload() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("scheduler-churn-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "ready_queue_ops":1000,
              "trigger_evaluations":800,
              "branch_propagations":300,
              "retry_events":100,
              "churn_budget_ops":3000
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(PerformanceCommands::SchedulerChurn { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::SchedulerChurn { simulation: simulation.clone() },
        )
        .expect("scheduler churn");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::scheduler_churn_payload(&simulation).expect("report");
        assert!(report.within_budget);
        assert!(!report.retry_storm_detected);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_scheduler_churn_flags_budget_overrun_and_retry_storm() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("scheduler-churn-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "ready_queue_ops":100,
              "trigger_evaluations":200,
              "branch_propagations":200,
              "retry_events":80,
              "churn_budget_ops":300
            }"#,
        )
        .expect("write simulation");
        let report = super::scheduler_churn_payload(&simulation).expect("report");
        assert!(!report.within_budget);
        assert!(report.retry_storm_detected);
        for expected in [
            "scheduler churn exceeds configured budget",
            "retry storm signal detected in scheduler workload",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn performance_artifact_write_profile_accepts_budgeted_write_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("artifact-write-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "small_write_count":1000,
              "large_write_count":30,
              "nested_dir_count":20,
              "bytes_written":104857600,
              "elapsed_ms":1500,
              "max_elapsed_ms":2000
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(PerformanceCommands::ArtifactWriteProfile { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::ArtifactWriteProfile { simulation: simulation.clone() },
        )
        .expect("artifact write profile");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::artifact_write_profile_payload(&simulation).expect("report");
        assert!(report.within_elapsed_budget);
        assert!(report.nested_layout_present);
        assert!(report.throughput_bytes_per_sec > 0.0);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_artifact_write_profile_flags_slow_or_shallow_coverage() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("artifact-write-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "small_write_count":0,
              "large_write_count":0,
              "nested_dir_count":0,
              "bytes_written":1024,
              "elapsed_ms":5000,
              "max_elapsed_ms":2000
            }"#,
        )
        .expect("write simulation");
        let report = super::artifact_write_profile_payload(&simulation).expect("report");
        assert!(!report.within_elapsed_budget);
        assert!(!report.nested_layout_present);
        for expected in [
            "artifact write elapsed time exceeds budget",
            "artifact write profile lacks nested directory coverage",
            "artifact write profile has no write operations",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn performance_memory_ceilings_accepts_bounded_phases() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("memory-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "phase_metrics_mb":{
                "parse":128,
                "canonicalize":192,
                "plan":256,
                "expansion":320,
                "run-history-import":200
              },
              "phase_ceilings_mb":{
                "parse":256,
                "canonicalize":256,
                "plan":300,
                "expansion":512,
                "run-history-import":256
              }
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(PerformanceCommands::MemoryCeilings { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::MemoryCeilings { simulation: simulation.clone() },
        )
        .expect("memory ceilings");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::memory_ceilings_payload(&simulation).expect("report");
        assert!(report.within_ceiling);
        assert!(report.over_ceiling_phases.is_empty());
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_memory_ceilings_flags_overages_and_missing_ceilings() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("memory-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "phase_metrics_mb":{
                "parse":512,
                "canonicalize":128,
                "plan":400
              },
              "phase_ceilings_mb":{
                "parse":256,
                "canonicalize":256
              }
            }"#,
        )
        .expect("write simulation");
        let report = super::memory_ceilings_payload(&simulation).expect("report");
        assert!(!report.within_ceiling);
        assert_eq!(report.over_ceiling_phases, vec!["parse".to_string()]);
        assert!(report.gaps.iter().any(|gap| gap == "parse exceeds memory ceiling"));
        assert!(report.gaps.iter().any(|gap| gap == "plan has no configured memory ceiling"));
    }

    #[test]
    fn performance_streaming_output_accepts_bounded_chunked_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("streaming-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "stream_name":"stdout",
              "total_bytes":10485760,
              "chunk_bytes":65536,
              "peak_in_memory_bytes":262144,
              "max_in_memory_bytes":1048576
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(PerformanceCommands::StreamingOutput { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::StreamingOutput { simulation: simulation.clone() },
        )
        .expect("streaming output");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::streaming_output_payload(&simulation).expect("report");
        assert!(report.bounded_memory);
        assert!(report.streaming_effective);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_streaming_output_flags_full_buffer_or_overflow() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("streaming-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "stream_name":"stderr",
              "total_bytes":1024,
              "chunk_bytes":1024,
              "peak_in_memory_bytes":8192,
              "max_in_memory_bytes":4096
            }"#,
        )
        .expect("write simulation");
        let report = super::streaming_output_payload(&simulation).expect("report");
        assert!(!report.bounded_memory);
        assert!(!report.streaming_effective);
        for expected in [
            "stream handling exceeded memory ceiling",
            "chunk size indicates non-streaming full-buffer behavior",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn performance_run_history_compaction_accepts_compacted_low_latency_history() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("history-compact-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "records_before":100000,
              "records_after":20000,
              "bytes_before":104857600,
              "bytes_after":15728640,
              "query_p95_before_ms":400,
              "query_p95_after_ms":120,
              "max_query_p95_ms":200
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(PerformanceCommands::RunHistoryCompaction { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::RunHistoryCompaction { simulation: simulation.clone() },
        )
        .expect("run history compaction");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::run_history_compaction_payload(&simulation).expect("report");
        assert!(report.query_within_budget);
        assert!(report.compaction_ratio < 1.0);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_run_history_compaction_flags_regressions() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("history-compact-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "records_before":1000,
              "records_after":1200,
              "bytes_before":2048,
              "bytes_after":4096,
              "query_p95_before_ms":200,
              "query_p95_after_ms":350,
              "max_query_p95_ms":300
            }"#,
        )
        .expect("write simulation");
        let report = super::run_history_compaction_payload(&simulation).expect("report");
        assert!(!report.query_within_budget);
        for expected in [
            "compaction increased run-history record count",
            "compaction increased run-history storage bytes",
            "post-compaction run-history query p95 exceeds budget",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn performance_benchmark_report_governance_accepts_complete_reproducible_report() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("benchmark-governance-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "fixture_id":"planner-large-v1",
              "hardware_note":"apple-m2-32gb",
              "run_version":"perf-2026-04-30",
              "variance_pct":3.2,
              "max_variance_pct":5.0,
              "reproducible":true
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(PerformanceCommands::BenchmarkReportGovernance {
            simulation: simulation.clone(),
        });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::BenchmarkReportGovernance { simulation: simulation.clone() },
        )
        .expect("benchmark report governance");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::benchmark_report_governance_payload(&simulation).expect("report");
        assert!(report.governance_passed);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn performance_benchmark_report_governance_flags_missing_metadata_and_variance_drift() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("benchmark-governance-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "fixture_id":"",
              "hardware_note":"",
              "run_version":"",
              "variance_pct":8.5,
              "max_variance_pct":5.0,
              "reproducible":false
            }"#,
        )
        .expect("write simulation");
        let report = super::benchmark_report_governance_payload(&simulation).expect("report");
        assert!(!report.governance_passed);
        for expected in [
            "benchmark report fixture identity is missing",
            "benchmark report hardware note is missing",
            "benchmark report version is missing",
            "benchmark report variance exceeds allowed budget",
            "benchmark run is not reproducible",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn performance_regression_gates_accepts_clean_or_approved_override_state() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let clean = dir.path().join("regression-clean.json");
        std::fs::write(
            &clean,
            r#"{
              "metrics":[
                {"metric":"graph-parse","baseline_ms":100,"current_ms":103},
                {"metric":"plan-lowering","baseline_ms":220,"current_ms":230}
              ],
              "max_regression_pct":8.0,
              "override_applied":false,
              "override_reason":null,
              "override_ticket":null
            }"#,
        )
        .expect("write simulation");
        let report = super::performance_regression_gates_payload(&clean).expect("report");
        assert!(report.gates_passed);
        assert!(report.failing_metrics.is_empty());

        let overridden = dir.path().join("regression-override.json");
        std::fs::write(
            &overridden,
            r#"{
              "metrics":[
                {"metric":"canonicalize","baseline_ms":100,"current_ms":140}
              ],
              "max_regression_pct":8.0,
              "override_applied":true,
              "override_reason":"known slowdown from integrity checks",
              "override_ticket":"PERF-102"
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(PerformanceCommands::PerformanceRegressionGates {
            simulation: overridden.clone(),
        });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::PerformanceRegressionGates { simulation: overridden.clone() },
        )
        .expect("performance regression gates");
        assert_eq!(code, ExitCode::SUCCESS);
        let override_report =
            super::performance_regression_gates_payload(&overridden).expect("override report");
        assert!(override_report.gates_passed);
        assert!(override_report.override_valid);
    }

    #[test]
    fn performance_regression_gates_flag_unapproved_or_unjustified_failures() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("regression-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "metrics":[
                {"metric":"graph-parse","baseline_ms":100,"current_ms":130}
              ],
              "max_regression_pct":8.0,
              "override_applied":true,
              "override_reason":"",
              "override_ticket":null
            }"#,
        )
        .expect("write simulation");
        let report = super::performance_regression_gates_payload(&simulation).expect("report");
        assert!(!report.gates_passed);
        assert!(!report.override_valid);
        assert_eq!(report.failing_metrics, vec!["graph-parse".to_string()]);
        assert!(report
            .gaps
            .iter()
            .any(|gap| gap == "performance regression override is missing reason or ticket"));
    }
}
