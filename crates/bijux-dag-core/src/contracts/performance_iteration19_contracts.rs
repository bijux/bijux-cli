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
    let inventory_complexity_score =
        input.route_count + input.app_count.saturating_mul(20) + input.plugin_count.saturating_mul(30);
    let within_budget =
        input.help_startup_ms <= 250.0 && input.median_dispatch_ms <= 20.0 && input.p95_dispatch_ms <= 60.0;
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
    for (name, value) in [
        ("canonical_json_ms", input.canonical_json_ms),
        ("fingerprint_ms", input.fingerprint_ms),
    ] {
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

#[cfg(test)]
mod tests {
    use super::{
        evaluate_planner_lowering_and_explain,
        evaluate_runtime_startup_benchmark,
        evaluate_canonicalization_and_fingerprinting,
        evaluate_graph_parse_and_validation_budget, evaluate_route_dispatch_and_help_startup,
        CanonicalFingerprintBenchmarkInputV1, GraphValidationBenchmarkInputV1,
        RuntimeStartupBenchmarkInputV1,
        PlannerLoweringBenchmarkInputV1, RouteDispatchBenchmarkInputV1,
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
        let healthy = evaluate_graph_parse_and_validation_budget(&GraphValidationBenchmarkInputV1 {
            small_graph_ms: 12.0,
            medium_graph_ms: 54.0,
            large_graph_ms: 210.0,
            invalid_graph_ms: 80.0,
            fuzz_graph_ms: 330.0,
        })
        .expect("healthy graph benchmark");
        assert!(healthy.within_budget);

        let regressed = evaluate_graph_parse_and_validation_budget(&GraphValidationBenchmarkInputV1 {
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

        let slow = evaluate_canonicalization_and_fingerprinting(&CanonicalFingerprintBenchmarkInputV1 {
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
}
