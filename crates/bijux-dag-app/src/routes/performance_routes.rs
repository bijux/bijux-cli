use crate::commands::{DagCli, PerformanceCommands};
use crate::{emit_json, ExitCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
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

fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
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
}
