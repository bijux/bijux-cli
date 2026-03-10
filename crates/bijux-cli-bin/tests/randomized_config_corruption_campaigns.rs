#![forbid(unsafe_code)]
//! Randomized config-corruption campaigns and invariants.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn bounded(&mut self, upper: usize) -> usize {
        if upper == 0 {
            0
        } else {
            (self.next_u64() as usize) % upper
        }
    }
}

#[derive(Clone, Copy)]
enum Mutator {
    Truncate,
    ByteFlip,
    DeleteLine,
    DuplicateLine,
    DuplicateKey,
    ExtraField,
}

fn temp_dir(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("bijux-config-campaign-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("mkdir temp");
    path
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn mutate_config(seed: &str, mutator: Mutator, rng: &mut Lcg) -> String {
    match mutator {
        Mutator::Truncate => {
            let keep = rng.bounded(seed.len().max(1));
            seed[..keep].to_owned()
        }
        Mutator::ByteFlip => {
            let mut bytes = seed.as_bytes().to_vec();
            if bytes.is_empty() {
                bytes.push(0);
            }
            let idx = rng.bounded(bytes.len());
            bytes[idx] ^= 0x7f;
            String::from_utf8_lossy(&bytes).into_owned()
        }
        Mutator::DeleteLine => {
            let mut lines: Vec<&str> = seed.lines().collect();
            if !lines.is_empty() {
                let idx = rng.bounded(lines.len());
                lines.remove(idx);
            }
            lines.join("\n")
        }
        Mutator::DuplicateLine => {
            let mut lines: Vec<String> = seed.lines().map(ToOwned::to_owned).collect();
            if !lines.is_empty() {
                let idx = rng.bounded(lines.len());
                let line = lines[idx].clone();
                lines.insert(idx, line);
            }
            lines.join("\n")
        }
        Mutator::DuplicateKey => {
            let mut out = seed.to_owned();
            out.push_str("BIJUXCLI_ALPHA=shadow\n");
            out
        }
        Mutator::ExtraField => {
            let mut out = seed.to_owned();
            out.push_str("NOT_A_CONFIG_ASSIGNMENT\n");
            out
        }
    }
}

fn assert_known_status(out: &Output, context: &str) {
    assert!(
        matches!(out.status.code(), Some(0) | Some(1) | Some(2)),
        "{context} produced unexpected status {:?}",
        out.status.code()
    );
}

#[test]
fn randomized_corruption_campaigns_cover_config_reads_writes_and_all_mutation_subcommands() {
    let root = temp_dir("full-campaign");
    let config = root.join("active.env");
    let load_source = root.join("source.env");
    let export_path = root.join("export.env");

    let mut rng = Lcg::new(0xA11CE1234u64);
    let seed = "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\nBIJUXCLI_GAMMA=3\n";
    let mutators = [
        Mutator::Truncate,
        Mutator::ByteFlip,
        Mutator::DeleteLine,
        Mutator::DuplicateLine,
        Mutator::DuplicateKey,
        Mutator::ExtraField,
    ];

    for mutator in mutators {
        let mutated = mutate_config(seed, mutator, &mut rng);
        fs::write(&config, &mutated).expect("write mutated config");
        fs::write(&load_source, &mutated).expect("write mutated source");

        let cfg = config.to_str().expect("utf-8");
        let src = load_source.to_str().expect("utf-8");
        let exp = export_path.to_str().expect("utf-8");

        assert_known_status(
            &run(&[
                "cli",
                "config",
                "list",
                "--format",
                "json",
                "--no-pretty",
                "--config-path",
                cfg,
            ]),
            "config list",
        );
        assert_known_status(
            &run(&[
                "cli",
                "config",
                "get",
                "alpha",
                "--format",
                "json",
                "--no-pretty",
                "--config-path",
                cfg,
            ]),
            "config get",
        );
        assert_known_status(
            &run(&["cli", "config", "set", "delta=4", "--config-path", cfg]),
            "config set",
        );
        assert_known_status(
            &run(&[
                "cli",
                "config",
                "unset",
                "alpha",
                "--format",
                "json",
                "--no-pretty",
                "--config-path",
                cfg,
            ]),
            "config unset",
        );
        assert_known_status(
            &run(&[
                "cli",
                "config",
                "clear",
                "--format",
                "json",
                "--no-pretty",
                "--config-path",
                cfg,
            ]),
            "config clear",
        );
        assert_known_status(
            &run(&[
                "cli",
                "config",
                "export",
                exp,
                "--format",
                "json",
                "--no-pretty",
                "--config-path",
                cfg,
            ]),
            "config export",
        );
        assert_known_status(
            &run(&["cli", "config", "load", src, "--config-path", cfg]),
            "config load",
        );
    }
}

#[test]
fn config_mutations_never_silently_destroy_unrelated_valid_keys() {
    let root = temp_dir("unrelated-keys");
    let config = root.join("active.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBIJUXCLI_STABLE=keep\n").expect("seed");

    let cfg = config.to_str().expect("utf-8");
    let out = run(&["cli", "config", "set", "bad-key=1", "--config-path", cfg]);
    assert_eq!(out.status.code(), Some(2));

    let body = fs::read_to_string(&config).expect("read config");
    assert!(body.contains("BIJUXCLI_STABLE=keep"));
}

#[test]
fn config_corruption_has_stable_failure_class_and_recovery_path() {
    let root = temp_dir("stable-failure-class");
    let config = root.join("active.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write broken");
    let cfg = config.to_str().expect("utf-8");

    let first =
        run(&["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg]);
    let second =
        run(&["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg]);
    assert_eq!(first.status.code(), second.status.code());

    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("repair");
    let repaired =
        run(&["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg]);
    assert_eq!(repaired.status.code(), Some(0));
}

#[test]
fn failed_config_load_rolls_back_and_preserves_coherent_state() {
    let root = temp_dir("rollback-coherent");
    let config = root.join("active.env");
    let source = root.join("source.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBIJUXCLI_STABLE=keep\n").expect("seed active");
    fs::write(&source, "BROKEN_LINE\n").expect("seed broken source");

    let before = fs::read_to_string(&config).expect("read before");
    let out = run(&[
        "cli",
        "config",
        "load",
        source.to_str().expect("utf-8"),
        "--config-path",
        config.to_str().expect("utf-8"),
    ]);
    assert_eq!(out.status.code(), Some(2));

    let after = fs::read_to_string(&config).expect("read after");
    assert_eq!(before, after);

    let list = run(&[
        "cli",
        "config",
        "list",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config.to_str().expect("utf-8"),
    ]);
    assert_eq!(list.status.code(), Some(0));
}

#[test]
fn state_doctor_reports_corruption_introduced_by_campaign_harness() {
    let root = temp_dir("doctor-detects");
    let config = root.join("active.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write broken");

    let out = run(&[
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config.to_str().expect("utf-8"),
    ]);
    assert_known_status(&out, "state doctor");
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json payload");
    let issues = payload["doctor"]["issues"].as_array().expect("issues array");
    assert!(issues.iter().any(|i| i["area"] == "config"));
}

#[test]
fn repeated_run_corruption_inputs_are_deterministic_for_config_command_set() {
    let root = temp_dir("repeat-determinism");
    let config = root.join("active.env");
    let source = root.join("source.env");
    fs::write(&config, "BROKEN_LINE\n").expect("broken active");
    fs::write(&source, "BROKEN_LINE\n").expect("broken source");

    let cfg = config.to_str().expect("utf-8");
    let src = source.to_str().expect("utf-8");
    let cases: [&[&str]; 8] = [
        &["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg],
        &["cli", "config", "get", "alpha", "--format", "json", "--no-pretty", "--config-path", cfg],
        &["cli", "config", "set", "delta=1", "--config-path", cfg],
        &[
            "cli",
            "config",
            "unset",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            cfg,
        ],
        &["cli", "config", "clear", "--format", "json", "--no-pretty", "--config-path", cfg],
        &["cli", "config", "export", src, "--format", "json", "--no-pretty", "--config-path", cfg],
        &["cli", "config", "load", src, "--config-path", cfg],
        &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty", "--config-path", cfg],
    ];

    for args in cases {
        let first = run(args);
        let second = run(args);
        assert_eq!(first.status.code(), second.status.code(), "exit drift for {args:?}");
    }
}
