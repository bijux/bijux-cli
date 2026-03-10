#![forbid(unsafe_code)]
//! Performance realism and regression budget coverage for high-value commands.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-performance-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run_once(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn average_duration_ms(args: &[&str], envs: &[(&str, String)], iterations: usize) -> u128 {
    let mut total = Duration::from_millis(0);
    let runs = iterations.max(1);
    for _ in 0..runs {
        let started = Instant::now();
        let output = run_once(args, envs);
        assert!(
            output.status.success(),
            "command failed for args {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        total += started.elapsed();
    }
    total.as_millis() / runs as u128
}

fn payload_size_bytes(args: &[&str], envs: &[(&str, String)]) -> usize {
    let output = run_once(args, envs);
    assert!(
        output.status.success(),
        "command failed for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout.len() + output.stderr.len()
}

#[test]
fn startup_benchmarks_for_key_commands_stay_within_budget() {
    let temp = temp_dir("startup-key");
    let config = temp.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("seed config");
    let config_path = config.display().to_string();

    let cases: Vec<(&[&str], Vec<(&str, String)>, usize, u128)> = vec![
        (&["version", "--format", "json", "--no-pretty"], vec![], 12, 120),
        (&["status", "--format", "json", "--no-pretty"], vec![], 8, 250),
        (&["doctor", "--format", "json", "--no-pretty"], vec![], 6, 500),
        (&["plugins", "list", "--format", "json", "--no-pretty"], vec![], 6, 400),
        (&["dev", "cli", "status", "--format", "json", "--no-pretty"], vec![], 4, 900),
    ];

    for (args, envs, iterations, budget_ms) in cases {
        let avg = average_duration_ms(args, &envs, iterations);
        assert!(
            avg <= budget_ms,
            "startup budget exceeded for {args:?}: avg={avg}ms budget={budget_ms}ms"
        );
    }

    let config_get_ms = average_duration_ms(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            &config_path,
        ],
        &[],
        10,
    );
    assert!(
        config_get_ms <= 200,
        "startup budget exceeded for cli config get: avg={config_get_ms}ms budget=200ms"
    );
}

#[test]
fn startup_benchmarks_under_registry_config_and_history_stress_stay_within_budget() {
    let temp = temp_dir("stress");

    let plugins_dir = temp.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{broken").expect("write broken registry");
    let broken_registry_ms = average_duration_ms(
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
        5,
    );
    assert!(
        broken_registry_ms <= 500,
        "broken registry startup budget exceeded: {broken_registry_ms}ms"
    );

    let huge_plugins_dir = temp.join("plugins-large");
    fs::create_dir_all(&huge_plugins_dir).expect("mkdir large plugins");
    let large_records: Vec<String> = (0..2_500)
        .map(|i| {
            format!("{{\"namespace\":\"p{i}\",\"entrypoint\":\"plugin{i}.py\",\"enabled\":true}}")
        })
        .collect();
    fs::write(huge_plugins_dir.join("registry.json"), format!("[{}]", large_records.join(",")))
        .expect("write large registry");
    let large_registry_ms = average_duration_ms(
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", huge_plugins_dir.display().to_string())],
        4,
    );
    assert!(
        large_registry_ms <= 900,
        "large registry startup budget exceeded: {large_registry_ms}ms"
    );

    let config_path = temp.join("large.env");
    let mut lines = String::new();
    for i in 0..12_000 {
        lines.push_str(&format!("BIJUXCLI_KEY_{i}={i}\n"));
    }
    lines.push_str("BIJUXCLI_ALPHA=1\n");
    fs::write(&config_path, lines).expect("write large config");
    let large_config_ms = average_duration_ms(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[],
        5,
    );
    assert!(large_config_ms <= 650, "large config startup budget exceeded: {large_config_ms}ms");

    let history_path = temp.join("large.history.json");
    let entries: Vec<String> =
        (0..20_000).map(|i| format!("{{\"command\":\"status\",\"timestamp\":{i}}}")).collect();
    fs::write(&history_path, format!("[{}]", entries.join(","))).expect("write large history");
    let large_history_ms = average_duration_ms(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
        4,
    );
    assert!(
        large_history_ms <= 1200,
        "large history startup budget exceeded: {large_history_ms}ms"
    );
}

#[test]
fn payload_size_benchmarks_for_key_commands_stay_within_budget() {
    let temp = temp_dir("payload-size");
    let plugins_dir = temp.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let version_bytes = payload_size_bytes(&["version", "--format", "json", "--no-pretty"], &[]);
    let status_bytes = payload_size_bytes(&["status", "--format", "json", "--no-pretty"], &[]);
    let plugins_list_bytes = payload_size_bytes(
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    );

    assert!(version_bytes <= 4 * 1024, "version payload budget exceeded: {version_bytes} bytes");
    assert!(status_bytes <= 24 * 1024, "status payload budget exceeded: {status_bytes} bytes");
    assert!(
        plugins_list_bytes <= 32 * 1024,
        "plugins list payload budget exceeded: {plugins_list_bytes} bytes"
    );
}
