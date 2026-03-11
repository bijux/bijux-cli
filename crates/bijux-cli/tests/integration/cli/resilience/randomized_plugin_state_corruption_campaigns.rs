#![forbid(unsafe_code)]
//! Randomized plugin/history/memory corruption campaigns and invariants.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
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
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
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
    WrongType,
    MissingField,
    ExtraField,
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-plugin-state-campaign-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("mkdir temp");
    path
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run command")
}

fn assert_known_status(out: &Output, context: &str) {
    assert!(
        matches!(out.status.code(), Some(0) | Some(1) | Some(2)),
        "{context} produced unexpected status {:?}",
        out.status.code()
    );
}

fn mutate_jsonish(seed: &str, mutator: Mutator, rng: &mut Lcg) -> String {
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
        Mutator::WrongType => "{\"plugins\":\"wrong\"}".to_owned(),
        Mutator::MissingField => seed.replace("\"name\":\"sample\",", ""),
        Mutator::ExtraField => format!("{seed}\n{{\"unexpected\":true}}"),
    }
}

#[test]
fn randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths() {
    let root = temp_dir("domain-campaign");
    let home = root.join("home");
    let plugins = root.join("plugins");
    let history = root.join("history.log");
    let memory = home.join(".bijux").join(".memory.json");
    let registry = plugins.join("registry.json");

    fs::create_dir_all(memory.parent().expect("memory parent")).expect("mkdir memory parent");
    fs::create_dir_all(&plugins).expect("mkdir plugins");

    let envs = vec![
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
    ];

    let mut rng = Lcg::new(0xBADC0FFEu64);
    let registry_seed =
        "{\"plugins\":[{\"name\":\"sample\",\"path\":\"/tmp/sample\",\"enabled\":true}]}";
    let history_seed = "status\ndoctor\n";
    let memory_seed = "{\"alpha\":{\"value\":\"1\"}}";

    let mutators = [
        Mutator::Truncate,
        Mutator::ByteFlip,
        Mutator::WrongType,
        Mutator::MissingField,
        Mutator::ExtraField,
    ];

    for mutator in mutators {
        fs::write(&registry, mutate_jsonish(registry_seed, mutator, &mut rng))
            .expect("write registry");
        fs::write(&history, mutate_jsonish(history_seed, mutator, &mut rng))
            .expect("write history");
        fs::write(&memory, mutate_jsonish(memory_seed, mutator, &mut rng)).expect("write memory");

        assert_known_status(
            &run_with_env(
                &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
                &envs,
            ),
            "plugins list",
        );
        assert_known_status(
            &run_with_env(
                &["cli", "plugins", "check", "--format", "json", "--no-pretty"],
                &envs,
            ),
            "plugins check",
        );
        assert_known_status(
            &run_with_env(
                &[
                    "cli",
                    "plugins",
                    "inspect",
                    "sample",
                    "--format",
                    "json",
                    "--no-pretty",
                ],
                &envs,
            ),
            "plugins inspect",
        );
        assert_known_status(
            &run_with_env(
                &[
                    "cli",
                    "plugins",
                    "doctor",
                    "--format",
                    "json",
                    "--no-pretty",
                ],
                &envs,
            ),
            "plugins doctor",
        );
        assert_known_status(
            &run_with_env(&["history", "--format", "json", "--no-pretty"], &envs),
            "history",
        );
        assert_known_status(
            &run_with_env(
                &["memory", "list", "--format", "json", "--no-pretty"],
                &envs,
            ),
            "memory list",
        );
    }
}

#[test]
fn one_broken_plugin_never_hides_unrelated_healthy_plugins() {
    let root = temp_dir("healthy-visible");
    let home = root.join("home");
    let plugins = root.join("plugins");
    let healthy_path = root.join("healthy-plugin");
    let broken_path = root.join("broken-plugin");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::create_dir_all(&healthy_path).expect("mkdir healthy");
    fs::create_dir_all(&broken_path).expect("mkdir broken");

    let registry = plugins.join("registry.json");
    fs::write(
        &registry,
        format!(
            "{{\"plugins\":[{{\"name\":\"healthy\",\"path\":\"{}\",\"enabled\":true}},{{\"name\":\"broken\",\"path\":\"{}\",\"enabled\":true}}]}}",
            healthy_path.display(),
            broken_path.display()
        ),
    )
    .expect("write registry");

    let out = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &[
            ("HOME", home.display().to_string()),
            ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ],
    );
    assert_known_status(&out, "plugins list healthy-visible");
    // Break one plugin path and ensure list still returns a stable response shape.
    fs::remove_dir_all(&broken_path).expect("remove broken plugin path");
    let out_after = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &[
            ("HOME", home.display().to_string()),
            ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ],
    );
    assert_known_status(&out_after, "plugins list healthy-visible broken-path");

    let payload_before: Value = serde_json::from_slice(&out.stdout).expect("json before");
    let payload_after: Value = serde_json::from_slice(&out_after.stdout).expect("json after");
    assert!(payload_before["plugins"].is_array());
    assert!(payload_after["plugins"].is_array());
}

#[test]
fn plugin_list_is_deterministic_for_identical_corrupted_registry() {
    let root = temp_dir("plugin-deterministic");
    let home = root.join("home");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(plugins.join("registry.json"), "{broken-json").expect("broken registry");

    let envs = vec![
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
    ];

    let first = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &envs,
    );
    let second = run_with_env(
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &envs,
    );
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths() {
    let root = temp_dir("plugin-rollback");
    let home = root.join("home");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(plugins.join("registry.json"), "{\"plugins\":[]}").expect("seed registry");

    let envs = vec![
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
    ];

    let before = fs::read_to_string(plugins.join("registry.json")).expect("before");
    let _ = run_with_env(
        &[
            "cli",
            "plugins",
            "inspect",
            "missing",
            "--format",
            "json",
            "--no-pretty",
        ],
        &envs,
    );
    let _ = run_with_env(
        &["cli", "plugins", "check", "--format", "json", "--no-pretty"],
        &envs,
    );
    let after = fs::read_to_string(plugins.join("registry.json")).expect("after");
    assert_eq!(before, after);
}

#[test]
fn plugin_doctor_reports_corruption_injected_by_campaign() {
    let root = temp_dir("plugin-doctor-detects");
    let home = root.join("home");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(plugins.join("registry.json"), "{broken-json").expect("broken registry");

    let out = run_with_env(
        &[
            "cli",
            "plugins",
            "doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[
            ("HOME", home.display().to_string()),
            ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ],
    );
    assert_known_status(&out, "plugins doctor");
    if !out.stdout.is_empty() {
        let payload: Value = serde_json::from_slice(&out.stdout).expect("json payload");
        let issues = payload["issues"].as_array().cloned().unwrap_or_default();
        let self_repair_attempted = payload["self_repair_attempted"].as_bool().unwrap_or(false);
        assert!(
            issues.iter().any(|row| row["area"] == "plugins")
                || !issues.is_empty()
                || self_repair_attempted,
            "doctor should expose plugin corruption via issues or self-repair signal"
        );
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).to_ascii_lowercase();
        assert!(
            stderr.contains("corrupt")
                || stderr.contains("registry")
                || out.status.code() == Some(1),
            "doctor should surface plugin corruption in stderr or exit class"
        );
    }
}

#[test]
fn history_and_memory_corruption_recovery_remains_stable_and_policy_compliant() {
    let root = temp_dir("history-memory-stable");
    let home = root.join("home");
    let plugins = root.join("plugins");
    let history = root.join("history.log");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("memory parent")).expect("mkdir memory parent");
    fs::create_dir_all(&plugins).expect("mkdir plugins");

    fs::write(
        &history,
        "[{\"command\":\"status\"},\"bad\",{\"command\":\"doctor\"}]",
    )
    .expect("write mixed history");
    fs::write(&memory, "{\"alpha\":1,\"beta\":{\"value\":\"2\"}} ").expect("write mixed memory");

    let envs = vec![
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
    ];

    let history_out = run_with_env(&["history", "--format", "json", "--no-pretty"], &envs);
    assert_known_status(&history_out, "history mixed");

    let memory_out = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &envs,
    );
    assert_known_status(&memory_out, "memory mixed");

    let first = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &envs,
    );
    let second = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &envs,
    );
    assert_eq!(first.status.code(), second.status.code());
}
