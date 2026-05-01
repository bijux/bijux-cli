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

#[cfg(test)]
mod tests {
    use super::{
        evaluate_graph_parse_and_validation_budget, evaluate_route_dispatch_and_help_startup,
        GraphValidationBenchmarkInputV1, RouteDispatchBenchmarkInputV1,
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
}
