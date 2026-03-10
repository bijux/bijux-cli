#![forbid(unsafe_code)]
//! Rendering performance realism benchmarks for large envelopes.

use std::time::{Duration, Instant};

use bijux_cli_core as _;
use bijux_cli_contracts::{ColorMode, LogLevel, OutputFormat};
use bijux_cli_output::{render_value, EmitterConfig};
use serde as _;
use serde_json::json;
use serde_yaml as _;
use thiserror as _;

fn large_payload(size: usize) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = (0..size)
        .map(|i| {
            json!({
                "index": i,
                "command": format!("cmd-{i}"),
                "status": if i % 3 == 0 { "complete" } else { "partial" },
                "latency_ms": i % 97,
                "message": "x".repeat(64),
            })
        })
        .collect();
    json!({"entries": rows, "summary": {"count": size}})
}

#[test]
fn large_json_rendering_stays_within_budget() {
    let payload = large_payload(3_000);
    let cfg = EmitterConfig {
        format: OutputFormat::Json,
        pretty: false,
        color: ColorMode::Never,
        log_level: LogLevel::Info,
        quiet: false,
        no_color: true,
    };

    let started = Instant::now();
    for _ in 0..8 {
        let rendered = render_value(&payload, cfg).expect("json render");
        assert!(rendered.starts_with('{'));
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "large json render budget exceeded: {elapsed:?}"
    );
}

#[test]
fn large_yaml_rendering_stays_within_budget() {
    let payload = large_payload(2_000);
    let cfg = EmitterConfig {
        format: OutputFormat::Yaml,
        pretty: true,
        color: ColorMode::Never,
        log_level: LogLevel::Info,
        quiet: false,
        no_color: true,
    };

    let started = Instant::now();
    for _ in 0..5 {
        let rendered = render_value(&payload, cfg).expect("yaml render");
        assert!(rendered.contains("entries:"));
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "large yaml render budget exceeded: {elapsed:?}"
    );
}
