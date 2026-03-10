#![forbid(unsafe_code)]
//! Performance sanity guard for config get baseline path.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow as _;
use bijux_cli_routing as _;
use bijux_cli_core::app::run_app;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_routing as _;
use clap as _;
use futures as _;
use serde_json as _;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-get-perf-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn config_get_repeated_invocation_stays_within_sanity_budget() {
    let temp = make_temp_dir("perf");
    let config_path = temp.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");

    let started = Instant::now();
    for _ in 0..200 {
        let out = run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "config".to_string(),
            "get".to_string(),
            "alpha".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-pretty".to_string(),
            "--config-path".to_string(),
            config_path.display().to_string(),
        ])
        .expect("run_app");
        assert_eq!(out.exit_code, 0);
    }

    assert!(
        started.elapsed() < Duration::from_secs(3),
        "config get performance budget exceeded: {:?}",
        started.elapsed()
    );
}
