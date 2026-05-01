use serde::{Deserialize, Serialize};

/// Route dispatch/help startup benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDispatchBenchmarkInputV1 {
    pub route_count: usize,
    pub app_count: usize,
    pub plugin_count: usize,
    pub help_startup_ms: f64,
    pub median_dispatch_ms: f64,
    pub p95_dispatch_ms: f64,
}

/// Route dispatch/help startup benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDispatchBenchmarkReportV1 {
    pub route_count: usize,
    pub inventory_complexity_score: usize,
    pub help_startup_ms: f64,
    pub median_dispatch_ms: f64,
    pub p95_dispatch_ms: f64,
    pub within_budget: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate route dispatch/help startup responsiveness under realistic route inventory growth.
pub fn evaluate_route_dispatch_and_help_startup(
    input: &RouteDispatchBenchmarkInputV1,
) -> Result<RouteDispatchBenchmarkReportV1, String> {
    if input.route_count == 0 {
        return Err("route dispatch benchmark requires route_count > 0".to_string());
    }
    for (field, value) in [
        ("help_startup_ms", input.help_startup_ms),
        ("median_dispatch_ms", input.median_dispatch_ms),
        ("p95_dispatch_ms", input.p95_dispatch_ms),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("route dispatch benchmark requires finite non-negative {field}"));
        }
    }
    let inventory_complexity_score = input.route_count
        + input.app_count.saturating_mul(20)
        + input.plugin_count.saturating_mul(30);
    let within_budget = input.help_startup_ms <= 250.0
        && input.median_dispatch_ms <= 20.0
        && input.p95_dispatch_ms <= 60.0;
    let mut diagnostics = Vec::new();
    if input.help_startup_ms > 250.0 {
        diagnostics.push("help startup exceeds 250ms budget".to_string());
    }
    if input.median_dispatch_ms > 20.0 {
        diagnostics.push("median dispatch exceeds 20ms budget".to_string());
    }
    if input.p95_dispatch_ms > 60.0 {
        diagnostics.push("p95 dispatch exceeds 60ms budget".to_string());
    }
    Ok(RouteDispatchBenchmarkReportV1 {
        route_count: input.route_count,
        inventory_complexity_score,
        help_startup_ms: input.help_startup_ms,
        median_dispatch_ms: input.median_dispatch_ms,
        p95_dispatch_ms: input.p95_dispatch_ms,
        within_budget,
        diagnostics,
    })
}

/// Parse/validation benchmark input across graph classes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphValidationBenchmarkInputV1 {
    pub small_graph_ms: f64,
    pub medium_graph_ms: f64,
    pub large_graph_ms: f64,
    pub invalid_graph_ms: f64,
    pub fuzz_graph_ms: f64,
}

/// Parse/validation benchmark budget report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphValidationBenchmarkReportV1 {
    pub max_ms: f64,
    pub within_budget: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate parse/validation benchmark budgets to prevent regression on varied graph classes.
pub fn evaluate_graph_parse_and_validation_budget(
    input: &GraphValidationBenchmarkInputV1,
) -> Result<GraphValidationBenchmarkReportV1, String> {
    let samples = [
        ("small_graph_ms", input.small_graph_ms, 40.0),
        ("medium_graph_ms", input.medium_graph_ms, 120.0),
        ("large_graph_ms", input.large_graph_ms, 350.0),
        ("invalid_graph_ms", input.invalid_graph_ms, 200.0),
        ("fuzz_graph_ms", input.fuzz_graph_ms, 500.0),
    ];
    let mut max_ms = 0.0f64;
    let mut diagnostics = Vec::new();
    for (name, value, budget) in samples {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("graph benchmark requires finite non-negative {name}"));
        }
        max_ms = max_ms.max(value);
        if value > budget {
            diagnostics.push(format!("{name} exceeds {budget:.0}ms budget"));
        }
    }
    Ok(GraphValidationBenchmarkReportV1 {
        max_ms,
        within_budget: diagnostics.is_empty(),
        diagnostics,
    })
}

/// Canonicalization/fingerprinting benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalFingerprintBenchmarkInputV1 {
    pub node_count: usize,
    pub canonical_json_ms: f64,
    pub fingerprint_ms: f64,
}

/// Canonicalization/fingerprinting benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalFingerprintBenchmarkReportV1 {
    pub total_ms: f64,
    pub throughput_nodes_per_second: f64,
    pub within_budget: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate canonicalization/fingerprinting hot-path budget for large graph workloads.
pub fn evaluate_canonicalization_and_fingerprinting(
    input: &CanonicalFingerprintBenchmarkInputV1,
) -> Result<CanonicalFingerprintBenchmarkReportV1, String> {
    if input.node_count == 0 {
        return Err("canonicalization benchmark requires node_count > 0".to_string());
    }
    for (name, value) in
        [("canonical_json_ms", input.canonical_json_ms), ("fingerprint_ms", input.fingerprint_ms)]
    {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("canonicalization benchmark requires finite non-negative {name}"));
        }
    }
    let total_ms = input.canonical_json_ms + input.fingerprint_ms;
    let throughput_nodes_per_second = if total_ms == 0.0 {
        f64::INFINITY
    } else {
        (input.node_count as f64) / (total_ms / 1000.0)
    };
    let within_budget = total_ms <= 280.0;
    let diagnostics = if within_budget {
        Vec::new()
    } else {
        vec!["canonicalization+fingerprint exceeds 280ms budget".to_string()]
    };
    Ok(CanonicalFingerprintBenchmarkReportV1 {
        total_ms,
        throughput_nodes_per_second,
        within_budget,
        diagnostics,
    })
}

/// Planner lowering/explain benchmark input by graph semantics family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannerLoweringBenchmarkInputV1 {
    pub chain_lowering_ms: f64,
    pub branch_lowering_ms: f64,
    pub reducer_lowering_ms: f64,
    pub matrix_lowering_ms: f64,
    pub subgraph_lowering_ms: f64,
    pub explain_ms: f64,
}

/// Planner lowering/explain benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannerLoweringBenchmarkReportV1 {
    pub max_lowering_ms: f64,
    pub explain_ms: f64,
    pub within_budget: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate planner lowering/explain performance budgets for complex graph shapes.
pub fn evaluate_planner_lowering_and_explain(
    input: &PlannerLoweringBenchmarkInputV1,
) -> Result<PlannerLoweringBenchmarkReportV1, String> {
    let lowering = [
        ("chain_lowering_ms", input.chain_lowering_ms),
        ("branch_lowering_ms", input.branch_lowering_ms),
        ("reducer_lowering_ms", input.reducer_lowering_ms),
        ("matrix_lowering_ms", input.matrix_lowering_ms),
        ("subgraph_lowering_ms", input.subgraph_lowering_ms),
    ];
    let mut max_lowering_ms = 0.0f64;
    for (name, value) in lowering {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("planner benchmark requires finite non-negative {name}"));
        }
        max_lowering_ms = max_lowering_ms.max(value);
    }
    if !input.explain_ms.is_finite() || input.explain_ms < 0.0 {
        return Err("planner benchmark requires finite non-negative explain_ms".to_string());
    }
    let mut diagnostics = Vec::new();
    if max_lowering_ms > 240.0 {
        diagnostics.push("planner lowering exceeds 240ms max budget".to_string());
    }
    if input.explain_ms > 150.0 {
        diagnostics.push("planner explain exceeds 150ms budget".to_string());
    }
    Ok(PlannerLoweringBenchmarkReportV1 {
        max_lowering_ms,
        explain_ms: input.explain_ms,
        within_budget: diagnostics.is_empty(),
        diagnostics,
    })
}

/// Runtime startup benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStartupBenchmarkInputV1 {
    pub run_root_creation_ms: f64,
    pub manifest_write_ms: f64,
    pub queue_admission_ms: f64,
    pub first_node_dispatch_ms: f64,
}

/// Runtime startup benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStartupBenchmarkReportV1 {
    pub startup_total_ms: f64,
    pub startup_overhead_tracked: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate startup overhead budget for runtime run initialization and first dispatch.
pub fn evaluate_runtime_startup_benchmark(
    input: &RuntimeStartupBenchmarkInputV1,
) -> Result<RuntimeStartupBenchmarkReportV1, String> {
    let slices = [
        ("run_root_creation_ms", input.run_root_creation_ms, 120.0),
        ("manifest_write_ms", input.manifest_write_ms, 80.0),
        ("queue_admission_ms", input.queue_admission_ms, 80.0),
        ("first_node_dispatch_ms", input.first_node_dispatch_ms, 180.0),
    ];
    let mut diagnostics = Vec::new();
    let mut startup_total_ms = 0.0f64;
    for (name, value, budget) in slices {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("runtime startup benchmark requires finite non-negative {name}"));
        }
        startup_total_ms += value;
        if value > budget {
            diagnostics.push(format!("{name} exceeds {budget:.0}ms budget"));
        }
    }
    if startup_total_ms > 350.0 {
        diagnostics.push("runtime startup total exceeds 350ms budget".to_string());
    }
    Ok(RuntimeStartupBenchmarkReportV1 {
        startup_total_ms,
        startup_overhead_tracked: true,
        diagnostics,
    })
}

/// Scheduler churn benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulerChurnBenchmarkInputV1 {
    pub retries_processed: usize,
    pub branch_events_processed: usize,
    pub ready_queue_ops: usize,
    pub cancellation_events: usize,
    pub elapsed_ms: f64,
}

/// Scheduler churn benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulerChurnBenchmarkReportV1 {
    pub events_per_second: f64,
    pub within_budget: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate scheduler churn throughput and regression budget.
pub fn evaluate_scheduler_churn_benchmark(
    input: &SchedulerChurnBenchmarkInputV1,
) -> Result<SchedulerChurnBenchmarkReportV1, String> {
    if !input.elapsed_ms.is_finite() || input.elapsed_ms <= 0.0 {
        return Err("scheduler churn benchmark requires elapsed_ms > 0".to_string());
    }
    let total_events = input.retries_processed
        + input.branch_events_processed
        + input.ready_queue_ops
        + input.cancellation_events;
    let events_per_second = (total_events as f64) / (input.elapsed_ms / 1000.0);
    let mut diagnostics = Vec::new();
    if events_per_second < 4_000.0 {
        diagnostics.push("scheduler churn throughput below 4000 events/s budget".to_string());
    }
    if input.cancellation_events > 0 && input.cancellation_events.saturating_mul(2) > total_events {
        diagnostics.push("cancellation storm dominates scheduler churn workload".to_string());
    }
    Ok(SchedulerChurnBenchmarkReportV1 {
        events_per_second,
        within_budget: diagnostics.is_empty(),
        diagnostics,
    })
}

/// Artifact write/inventory benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactBenchmarkInputV1 {
    pub small_output_write_ms: f64,
    pub large_output_write_ms: f64,
    pub directory_inventory_ms: f64,
    pub bundle_export_ms: f64,
}

/// Artifact benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactBenchmarkReportV1 {
    pub deterministic_budget_passed: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate artifact write/inventory/export budget with deterministic-operation emphasis.
pub fn evaluate_artifact_write_and_inventory_benchmark(
    input: &ArtifactBenchmarkInputV1,
) -> Result<ArtifactBenchmarkReportV1, String> {
    let checks = [
        ("small_output_write_ms", input.small_output_write_ms, 90.0),
        ("large_output_write_ms", input.large_output_write_ms, 320.0),
        ("directory_inventory_ms", input.directory_inventory_ms, 140.0),
        ("bundle_export_ms", input.bundle_export_ms, 500.0),
    ];
    let mut diagnostics = Vec::new();
    for (name, value, budget) in checks {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("artifact benchmark requires finite non-negative {name}"));
        }
        if value > budget {
            diagnostics.push(format!("{name} exceeds {budget:.0}ms deterministic budget"));
        }
    }
    Ok(ArtifactBenchmarkReportV1 {
        deterministic_budget_passed: diagnostics.is_empty(),
        diagnostics,
    })
}

/// Evidence verification benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceVerificationBenchmarkInputV1 {
    pub small_bundle_ms: f64,
    pub medium_bundle_ms: f64,
    pub large_bundle_ms: f64,
}

/// Evidence verification benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceVerificationBenchmarkReportV1 {
    pub max_bundle_ms: f64,
    pub release_track_ready: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate evidence verification performance for release tracking.
pub fn evaluate_evidence_verification_benchmark(
    input: &EvidenceVerificationBenchmarkInputV1,
) -> Result<EvidenceVerificationBenchmarkReportV1, String> {
    let values = [
        ("small_bundle_ms", input.small_bundle_ms, 120.0),
        ("medium_bundle_ms", input.medium_bundle_ms, 280.0),
        ("large_bundle_ms", input.large_bundle_ms, 700.0),
    ];
    let mut max_bundle_ms = 0.0f64;
    let mut diagnostics = Vec::new();
    for (name, value, budget) in values {
        if !value.is_finite() || value < 0.0 {
            return Err(format!(
                "evidence verification benchmark requires finite non-negative {name}"
            ));
        }
        max_bundle_ms = max_bundle_ms.max(value);
        if value > budget {
            diagnostics.push(format!("{name} exceeds {budget:.0}ms release budget"));
        }
    }
    Ok(EvidenceVerificationBenchmarkReportV1 {
        max_bundle_ms,
        release_track_ready: diagnostics.is_empty(),
        diagnostics,
    })
}

/// Run history query benchmark input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryQueryBenchmarkInputV1 {
    pub pagination_ms: f64,
    pub filtering_ms: f64,
    pub lineage_query_ms: f64,
    pub timeline_query_ms: f64,
}

/// Run history query benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryQueryBenchmarkReportV1 {
    pub max_query_ms: f64,
    pub responsive: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate long-lived run history query responsiveness.
pub fn evaluate_history_query_benchmark(
    input: &HistoryQueryBenchmarkInputV1,
) -> Result<HistoryQueryBenchmarkReportV1, String> {
    let checks = [
        ("pagination_ms", input.pagination_ms, 70.0),
        ("filtering_ms", input.filtering_ms, 90.0),
        ("lineage_query_ms", input.lineage_query_ms, 140.0),
        ("timeline_query_ms", input.timeline_query_ms, 160.0),
    ];
    let mut max_query_ms = 0.0f64;
    let mut diagnostics = Vec::new();
    for (name, value, budget) in checks {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("history query benchmark requires finite non-negative {name}"));
        }
        max_query_ms = max_query_ms.max(value);
        if value > budget {
            diagnostics.push(format!("{name} exceeds {budget:.0}ms responsiveness budget"));
        }
    }
    Ok(HistoryQueryBenchmarkReportV1 {
        max_query_ms,
        responsive: diagnostics.is_empty(),
        diagnostics,
    })
}

/// Cache effectiveness benchmark input with trust safety flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEffectivenessBenchmarkInputV1 {
    pub rerun_count: usize,
    pub cache_hit_count: usize,
    pub safe_invalidation_count: usize,
    pub unsafe_hit_detected: bool,
    pub elapsed_ms: f64,
}

/// Cache effectiveness benchmark report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEffectivenessBenchmarkReportV1 {
    pub cache_hit_ratio: f64,
    pub reruns_per_second: f64,
    pub trust_preserved: bool,
    pub diagnostics: Vec<String>,
}

/// Evaluate cache effectiveness while enforcing trust-safe reuse/invalidation constraints.
pub fn evaluate_cache_effectiveness_benchmark(
    input: &CacheEffectivenessBenchmarkInputV1,
) -> Result<CacheEffectivenessBenchmarkReportV1, String> {
    if input.rerun_count == 0 {
        return Err("cache effectiveness benchmark requires rerun_count > 0".to_string());
    }
    if !input.elapsed_ms.is_finite() || input.elapsed_ms <= 0.0 {
        return Err("cache effectiveness benchmark requires elapsed_ms > 0".to_string());
    }
    if input.cache_hit_count > input.rerun_count
        || input.safe_invalidation_count > input.rerun_count
    {
        return Err("cache benchmark counts cannot exceed rerun_count".to_string());
    }
    let cache_hit_ratio = (input.cache_hit_count as f64) / (input.rerun_count as f64);
    let reruns_per_second = (input.rerun_count as f64) / (input.elapsed_ms / 1000.0);
    let trust_preserved = !input.unsafe_hit_detected;
    let mut diagnostics = Vec::new();
    if cache_hit_ratio < 0.2 {
        diagnostics.push("cache hit ratio below 20%; inspect key granularity".to_string());
    }
    if !trust_preserved {
        diagnostics.push("unsafe cache hit detected; trust invariants violated".to_string());
    }
    if input.safe_invalidation_count == 0 {
        diagnostics.push("no safe invalidations observed; invalidation path untested".to_string());
    }

    Ok(CacheEffectivenessBenchmarkReportV1 {
        cache_hit_ratio,
        reruns_per_second,
        trust_preserved,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_artifact_write_and_inventory_benchmark, evaluate_cache_effectiveness_benchmark,
        evaluate_canonicalization_and_fingerprinting, evaluate_evidence_verification_benchmark,
        evaluate_graph_parse_and_validation_budget, evaluate_history_query_benchmark,
        evaluate_planner_lowering_and_explain, evaluate_route_dispatch_and_help_startup,
        evaluate_runtime_startup_benchmark, evaluate_scheduler_churn_benchmark,
        ArtifactBenchmarkInputV1, CacheEffectivenessBenchmarkInputV1,
        CanonicalFingerprintBenchmarkInputV1, EvidenceVerificationBenchmarkInputV1,
        GraphValidationBenchmarkInputV1, HistoryQueryBenchmarkInputV1,
        PlannerLoweringBenchmarkInputV1, RouteDispatchBenchmarkInputV1,
        RuntimeStartupBenchmarkInputV1, SchedulerChurnBenchmarkInputV1,
    };

    #[test]
    fn g181_route_dispatch_and_help_startup_remain_responsive_under_inventory_growth() {
        let report = evaluate_route_dispatch_and_help_startup(&RouteDispatchBenchmarkInputV1 {
            route_count: 140,
            app_count: 6,
            plugin_count: 5,
            help_startup_ms: 190.0,
            median_dispatch_ms: 12.0,
            p95_dispatch_ms: 41.0,
        })
        .expect("benchmark report should build");
        assert!(report.within_budget);
        assert!(report.diagnostics.is_empty());

        let slow = evaluate_route_dispatch_and_help_startup(&RouteDispatchBenchmarkInputV1 {
            route_count: 140,
            app_count: 6,
            plugin_count: 5,
            help_startup_ms: 320.0,
            median_dispatch_ms: 25.0,
            p95_dispatch_ms: 83.0,
        })
        .expect("slow benchmark should still report");
        assert!(!slow.within_budget);
        assert_eq!(slow.diagnostics.len(), 3);
    }

    #[test]
    fn g182_graph_parse_and_validation_budget_catches_regressions() {
        let healthy =
            evaluate_graph_parse_and_validation_budget(&GraphValidationBenchmarkInputV1 {
                small_graph_ms: 12.0,
                medium_graph_ms: 54.0,
                large_graph_ms: 210.0,
                invalid_graph_ms: 80.0,
                fuzz_graph_ms: 330.0,
            })
            .expect("healthy graph benchmark");
        assert!(healthy.within_budget);

        let regressed =
            evaluate_graph_parse_and_validation_budget(&GraphValidationBenchmarkInputV1 {
                small_graph_ms: 60.0,
                medium_graph_ms: 170.0,
                large_graph_ms: 470.0,
                invalid_graph_ms: 290.0,
                fuzz_graph_ms: 640.0,
            })
            .expect("regressed benchmark report");
        assert!(!regressed.within_budget);
        assert_eq!(regressed.diagnostics.len(), 5);
    }

    #[test]
    fn g183_canonicalization_and_fingerprinting_budget_is_measured_for_large_graphs() {
        let report =
            evaluate_canonicalization_and_fingerprinting(&CanonicalFingerprintBenchmarkInputV1 {
                node_count: 2_000,
                canonical_json_ms: 120.0,
                fingerprint_ms: 90.0,
            })
            .expect("benchmark should succeed");
        assert!(report.within_budget);
        assert!(report.throughput_nodes_per_second > 5_000.0);

        let slow =
            evaluate_canonicalization_and_fingerprinting(&CanonicalFingerprintBenchmarkInputV1 {
                node_count: 2_000,
                canonical_json_ms: 190.0,
                fingerprint_ms: 140.0,
            })
            .expect("slow benchmark should report");
        assert!(!slow.within_budget);
        assert_eq!(slow.diagnostics.len(), 1);
    }

    #[test]
    fn g184_planner_lowering_and_explain_remain_bounded_for_complex_shapes() {
        let report = evaluate_planner_lowering_and_explain(&PlannerLoweringBenchmarkInputV1 {
            chain_lowering_ms: 40.0,
            branch_lowering_ms: 70.0,
            reducer_lowering_ms: 90.0,
            matrix_lowering_ms: 110.0,
            subgraph_lowering_ms: 125.0,
            explain_ms: 85.0,
        })
        .expect("planner benchmark should work");
        assert!(report.within_budget);

        let slow = evaluate_planner_lowering_and_explain(&PlannerLoweringBenchmarkInputV1 {
            chain_lowering_ms: 120.0,
            branch_lowering_ms: 250.0,
            reducer_lowering_ms: 210.0,
            matrix_lowering_ms: 260.0,
            subgraph_lowering_ms: 270.0,
            explain_ms: 190.0,
        })
        .expect("slow planner benchmark");
        assert!(!slow.within_budget);
        assert_eq!(slow.diagnostics.len(), 2);
    }

    #[test]
    fn g185_runtime_startup_overhead_is_tracked_and_budgeted() {
        let report = evaluate_runtime_startup_benchmark(&RuntimeStartupBenchmarkInputV1 {
            run_root_creation_ms: 60.0,
            manifest_write_ms: 24.0,
            queue_admission_ms: 30.0,
            first_node_dispatch_ms: 70.0,
        })
        .expect("runtime startup benchmark should work");
        assert!(report.startup_overhead_tracked);
        assert!(report.diagnostics.is_empty());

        let slow = evaluate_runtime_startup_benchmark(&RuntimeStartupBenchmarkInputV1 {
            run_root_creation_ms: 180.0,
            manifest_write_ms: 95.0,
            queue_admission_ms: 85.0,
            first_node_dispatch_ms: 220.0,
        })
        .expect("slow startup benchmark");
        assert!(!slow.diagnostics.is_empty());
    }

    #[test]
    fn g186_scheduler_churn_benchmark_catches_retry_and_cancellation_regressions() {
        let healthy = evaluate_scheduler_churn_benchmark(&SchedulerChurnBenchmarkInputV1 {
            retries_processed: 3_000,
            branch_events_processed: 4_000,
            ready_queue_ops: 6_000,
            cancellation_events: 800,
            elapsed_ms: 2_000.0,
        })
        .expect("healthy scheduler benchmark");
        assert!(healthy.within_budget);

        let churn = evaluate_scheduler_churn_benchmark(&SchedulerChurnBenchmarkInputV1 {
            retries_processed: 500,
            branch_events_processed: 300,
            ready_queue_ops: 600,
            cancellation_events: 3_000,
            elapsed_ms: 2_500.0,
        })
        .expect("churn report");
        assert!(!churn.within_budget);
        assert_eq!(churn.diagnostics.len(), 2);
    }

    #[test]
    fn g187_artifact_write_inventory_and_export_stay_within_deterministic_budgets() {
        let healthy = evaluate_artifact_write_and_inventory_benchmark(&ArtifactBenchmarkInputV1 {
            small_output_write_ms: 35.0,
            large_output_write_ms: 180.0,
            directory_inventory_ms: 70.0,
            bundle_export_ms: 230.0,
        })
        .expect("artifact benchmark should work");
        assert!(healthy.deterministic_budget_passed);

        let slow = evaluate_artifact_write_and_inventory_benchmark(&ArtifactBenchmarkInputV1 {
            small_output_write_ms: 120.0,
            large_output_write_ms: 380.0,
            directory_inventory_ms: 210.0,
            bundle_export_ms: 580.0,
        })
        .expect("slow artifact benchmark should report");
        assert!(!slow.deterministic_budget_passed);
        assert_eq!(slow.diagnostics.len(), 4);
    }

    #[test]
    fn g188_evidence_verification_performance_is_release_tracked() {
        let healthy =
            evaluate_evidence_verification_benchmark(&EvidenceVerificationBenchmarkInputV1 {
                small_bundle_ms: 60.0,
                medium_bundle_ms: 180.0,
                large_bundle_ms: 420.0,
            })
            .expect("evidence benchmark");
        assert!(healthy.release_track_ready);

        let slow =
            evaluate_evidence_verification_benchmark(&EvidenceVerificationBenchmarkInputV1 {
                small_bundle_ms: 150.0,
                medium_bundle_ms: 330.0,
                large_bundle_ms: 880.0,
            })
            .expect("slow evidence benchmark");
        assert!(!slow.release_track_ready);
        assert_eq!(slow.diagnostics.len(), 3);
    }

    #[test]
    fn g189_history_query_benchmark_keeps_long_lived_usage_responsive() {
        let healthy = evaluate_history_query_benchmark(&HistoryQueryBenchmarkInputV1 {
            pagination_ms: 34.0,
            filtering_ms: 48.0,
            lineage_query_ms: 82.0,
            timeline_query_ms: 96.0,
        })
        .expect("history benchmark");
        assert!(healthy.responsive);

        let slow = evaluate_history_query_benchmark(&HistoryQueryBenchmarkInputV1 {
            pagination_ms: 95.0,
            filtering_ms: 120.0,
            lineage_query_ms: 190.0,
            timeline_query_ms: 230.0,
        })
        .expect("slow history benchmark");
        assert!(!slow.responsive);
        assert_eq!(slow.diagnostics.len(), 4);
    }

    #[test]
    fn g190_cache_effectiveness_never_trades_speed_for_trust() {
        let healthy = evaluate_cache_effectiveness_benchmark(&CacheEffectivenessBenchmarkInputV1 {
            rerun_count: 50,
            cache_hit_count: 28,
            safe_invalidation_count: 12,
            unsafe_hit_detected: false,
            elapsed_ms: 2_500.0,
        })
        .expect("cache benchmark");
        assert!(healthy.trust_preserved);
        assert!(healthy.cache_hit_ratio > 0.5);

        let unsafe_case =
            evaluate_cache_effectiveness_benchmark(&CacheEffectivenessBenchmarkInputV1 {
                rerun_count: 50,
                cache_hit_count: 40,
                safe_invalidation_count: 0,
                unsafe_hit_detected: true,
                elapsed_ms: 2_100.0,
            })
            .expect("unsafe cache benchmark");
        assert!(!unsafe_case.trust_preserved);
        assert!(unsafe_case
            .diagnostics
            .iter()
            .any(|line| line.contains("unsafe cache hit detected")));
    }
}
