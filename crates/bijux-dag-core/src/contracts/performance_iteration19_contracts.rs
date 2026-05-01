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

#[cfg(test)]
mod tests {
    use super::{evaluate_route_dispatch_and_help_startup, RouteDispatchBenchmarkInputV1};

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
}
