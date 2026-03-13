#![forbid(unsafe_code)]
//! Performance realism and regression budget coverage for high-value commands.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bijux_cli as _;
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
    let mut last_not_found: Option<std::io::Error> = None;

    for attempt in 0..8 {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
        cmd.args(args);
        for (k, v) in envs {
            cmd.env(k, v);
        }

        match cmd.output() {
            Ok(output) => return output,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                last_not_found = Some(err);
                if attempt < 7 {
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
            }
            Err(err) => panic!("binary should execute for args {args:?}: {err}"),
        }
    }

    let err = last_not_found.unwrap_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "bijux binary not found")
    });
    panic!("binary should execute for args {args:?}: {err}");
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
    let plugin_list_args = ["plugins", "list", "--format", "json", "--no-pretty"];
    let empty_registry_ms = average_duration_ms(
        &plugin_list_args,
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
        5,
    );

    fs::write(plugins_dir.join("registry.json"), "{broken").expect("write broken registry");
    let broken_registry_ms = average_duration_ms(
        &plugin_list_args,
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
        5,
    );
    assert!(
        broken_registry_ms <= empty_registry_ms + 1200,
        "broken registry startup budget exceeded: baseline={empty_registry_ms}ms stressed={broken_registry_ms}ms"
    );

    let huge_plugins_dir = temp.join("plugins-large");
    fs::create_dir_all(&huge_plugins_dir).expect("mkdir large plugins");
    let empty_large_registry_ms = average_duration_ms(
        &plugin_list_args,
        &[("BIJUXCLI_PLUGINS_DIR", huge_plugins_dir.display().to_string())],
        4,
    );
    let large_records: Vec<String> = (0..2_500)
        .map(|i| {
            format!("{{\"namespace\":\"p{i}\",\"entrypoint\":\"plugin{i}.py\",\"enabled\":true}}")
        })
        .collect();
    fs::write(huge_plugins_dir.join("registry.json"), format!("[{}]", large_records.join(",")))
        .expect("write large registry");
    let large_registry_ms = average_duration_ms(
        &plugin_list_args,
        &[("BIJUXCLI_PLUGINS_DIR", huge_plugins_dir.display().to_string())],
        4,
    );
    assert!(
        large_registry_ms <= empty_large_registry_ms + 1800,
        "large registry startup budget exceeded: baseline={empty_large_registry_ms}ms stressed={large_registry_ms}ms"
    );

    let small_config_path = temp.join("small.env");
    fs::write(&small_config_path, "BIJUXCLI_ALPHA=1\n").expect("write small config");
    let small_config_ms = average_duration_ms(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            small_config_path.to_str().expect("utf-8"),
        ],
        &[],
        5,
    );

    let config_path = temp.join("large.env");
    let mut lines = String::new();
    for i in 0..6_000 {
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
    let large_config_budget_ms = small_config_ms.saturating_mul(3) + 1200;
    assert!(
        large_config_ms <= large_config_budget_ms,
        "large config startup budget exceeded: baseline={small_config_ms}ms stressed={large_config_ms}ms budget={large_config_budget_ms}ms"
    );

    let baseline_history_path = temp.join("small.history.json");
    fs::write(&baseline_history_path, "[{\"command\":\"status\",\"timestamp\":1}]")
        .expect("write baseline history");
    let baseline_history_ms = average_duration_ms(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", baseline_history_path.display().to_string())],
        4,
    );

    let history_path = temp.join("large.history.json");
    let entries: Vec<String> =
        (0..20_000).map(|i| format!("{{\"command\":\"status\",\"timestamp\":{i}}}")).collect();
    fs::write(&history_path, format!("[{}]", entries.join(","))).expect("write large history");
    let large_history_ms = average_duration_ms(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
        4,
    );
    let large_history_budget_ms = (baseline_history_ms + 2500).max(3200);
    assert!(
        large_history_ms <= large_history_budget_ms,
        "large history startup budget exceeded: baseline={baseline_history_ms}ms stressed={large_history_ms}ms budget={large_history_budget_ms}ms"
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

#[test]
fn non_plugin_commands_do_not_scale_with_large_plugin_registries() {
    let temp = temp_dir("non-plugin-registry-isolation");
    let plugins_dir = temp.join("plugins-large");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let large_records: Vec<String> = (0..4_000)
        .map(|i| {
            format!("{{\"namespace\":\"p{i}\",\"entrypoint\":\"plugin{i}.py\",\"enabled\":true}}")
        })
        .collect();
    fs::write(plugins_dir.join("registry.json"), format!("[{}]", large_records.join(",")))
        .expect("write large registry");

    let config = temp.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("seed config");
    let history = temp.join("history.json");
    fs::write(&history, "[{\"command\":\"status\",\"timestamp\":1}]").expect("seed history");

    let config_args = [
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config.to_str().expect("utf-8"),
    ];
    let history_args = ["history", "--format", "json", "--no-pretty", "--limit", "1"];

    let base_config_ms = average_duration_ms(&config_args, &[], 5);
    let stressed_config_ms = average_duration_ms(
        &config_args,
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
        5,
    );
    assert!(
        stressed_config_ms <= base_config_ms.saturating_mul(3) + 80,
        "config get should not scale with plugin registry size: base={base_config_ms}ms stressed={stressed_config_ms}ms"
    );

    let base_history_ms = average_duration_ms(
        &history_args,
        &[("BIJUXCLI_HISTORY_FILE", history.display().to_string())],
        5,
    );
    let stressed_history_ms = average_duration_ms(
        &history_args,
        &[
            ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
            ("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string()),
        ],
        5,
    );
    assert!(
        stressed_history_ms <= base_history_ms.saturating_mul(3) + 80,
        "history should not scale with plugin registry size: base={base_history_ms}ms stressed={stressed_history_ms}ms"
    );
}

#[test]
fn large_history_with_small_limit_stays_within_startup_budget() {
    let temp = temp_dir("history-limit-budget");
    let history = temp.join("large.history.json");
    let entries: Vec<String> =
        (0..20_000).map(|i| format!("{{\"command\":\"status\",\"timestamp\":{i}}}")).collect();
    fs::write(&history, format!("[{}]", entries.join(","))).expect("write large history");

    let args = ["history", "--format", "json", "--no-pretty", "--limit", "1"];
    let envs = [("BIJUXCLI_HISTORY_FILE", history.display().to_string())];
    let avg_ms = average_duration_ms(&args, &envs, 4);
    assert!(avg_ms <= 900, "history --limit startup budget exceeded: {avg_ms}ms");

    let output = run_once(&args, &envs);
    assert!(output.status.success(), "history --limit should succeed");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json payload");
    let rows = payload["entries"].as_array().expect("entries array");
    assert_eq!(rows.len(), 1, "history --limit should return one entry");
}
