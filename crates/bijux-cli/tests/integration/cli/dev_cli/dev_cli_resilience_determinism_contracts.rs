#![forbid(unsafe_code)]
//! Failure-injection, determinism, and side-effect contracts for dev-cli control-plane commands.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn run(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.current_dir(workspace_root());
    cmd.output().expect("run command")
}

fn run_json(args: &[&str], envs: &[(&str, String)]) -> (i32, Option<Value>, String) {
    let out = run(args, envs);
    let code = out.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let parsed = serde_json::from_slice::<Value>(&out.stdout).ok();
    (code, parsed, stdout)
}

fn seed_state_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join(format!("bijux-dev-cli-resilience-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("plugins")).expect("mkdir");
    fs::write(root.join("config.env"), "BIJUXCLI_SAMPLE=1\n").expect("config");
    fs::write(root.join("history.json"), "[]").expect("history");
    fs::write(root.join("memory.json"), "{}").expect("memory");
    fs::write(
        root.join("plugins").join("healthy.toml"),
        "[plugin]\nname='healthy'\nentry='plugin:main'\n",
    )
    .expect("plugin");
    root
}

fn file_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).expect("read bytes")
}

#[test]
fn failure_injection_scenarios_have_stable_exit_class_and_no_panic() {
    let root = seed_state_root("failure-injection");
    let unreadable = root.join("unreadable.json");
    fs::write(&unreadable, "{\"k\":1}\n").expect("write unreadable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&unreadable).expect("meta").permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&unreadable, perms).expect("chmod");
    }
    let bad_json = root.join("broken.json");
    fs::write(&bad_json, "{bad-json").expect("write bad json");
    let missing_contracts_dir = root.join("missing-contracts");

    let unreadable_string = unreadable.to_string_lossy().to_string();
    let bad_json_string = bad_json.to_string_lossy().to_string();
    let cases: Vec<(&str, Vec<String>, Vec<(&str, String)>)> = vec![
        (
            "status-unreadable-artifact",
            vec!["dev", "cli", "status", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![("BIJUX_HISTORY_PATH", unreadable_string.clone())],
        ),
        (
            "parity-broken-json",
            vec!["dev", "cli", "parity", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![("BIJUX_MEMORY_PATH", bad_json_string.clone())],
        ),
        (
            "contracts-missing-snapshot",
            vec!["dev", "cli", "contracts", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![("PWD", missing_contracts_dir.to_string_lossy().to_string())],
        ),
        (
            "registry-unreadable-plugin-registry",
            vec!["dev", "cli", "registry", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![("BIJUX_PLUGINS_DIR", unreadable_string.clone())],
        ),
        (
            "env-unreadable-config",
            vec![
                "dev",
                "cli",
                "env",
                "--config-path",
                unreadable_string.as_str(),
                "--format",
                "json",
                "--no-pretty",
            ]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
            vec![],
        ),
        (
            "state-audit-missing-state-dir",
            vec!["dev", "cli", "state-audit", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![
                (
                    "BIJUX_HISTORY_PATH",
                    root.join("no-dir").join("history.json").to_string_lossy().to_string(),
                ),
                (
                    "BIJUX_MEMORY_PATH",
                    root.join("no-dir").join("memory.json").to_string_lossy().to_string(),
                ),
            ],
        ),
        (
            "state-doctor-corrupted-state",
            vec!["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![
                ("BIJUX_HISTORY_PATH", bad_json_string.clone()),
                ("BIJUX_MEMORY_PATH", bad_json_string.clone()),
            ],
        ),
        (
            "runtime-identity-path-ambiguity",
            vec!["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![(
                "PATH",
                format!("/tmp/bijux-a:/tmp/bijux-b:{}", std::env::var("PATH").unwrap_or_default()),
            )],
        ),
        (
            "package-health-metadata-mismatch",
            vec!["dev", "cli", "package-health", "--format", "json", "--no-pretty"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            vec![
                ("BIJUX_WHEEL_VERSION", "0.0.1".to_string()),
                ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string()),
            ],
        ),
    ];

    for (name, args, envs) in cases {
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let first = run_json(&arg_refs, &envs);
        let second = run_json(&arg_refs, &envs);
        assert_eq!(first.0, second.0, "exit class drift for {name}");
        if let Some(parsed) = first.1 {
            assert!(parsed.is_object(), "json should remain object for {name}");
        } else {
            assert!(
                !first.2.trim().is_empty(),
                "non-json outputs must still be non-empty for {name}"
            );
        }
    }
}

#[test]
fn summary_commands_are_deterministic_across_repeated_runs() {
    let commands = [
        ["dev", "cli", "status"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ];
    for command in commands {
        let mut args = command.to_vec();
        args.push("--format");
        args.push("json");
        args.push("--no-pretty");
        let first = run(&args, &[]);
        let second = run(&args, &[]);
        assert!(first.status.success(), "first run failed for {:?}", args);
        assert!(second.status.success(), "second run failed for {:?}", args);
        assert_eq!(first.stdout, second.stdout, "summary output drift for {:?}", args);
    }
}

#[test]
fn machine_readable_commands_are_deterministic_across_repeated_runs() {
    let commands: Vec<Vec<&str>> = vec![
        vec!["dev", "cli", "parity"],
        vec!["dev", "cli", "evidence", "audit"],
        vec!["dev", "cli", "routes"],
        vec!["dev", "cli", "registry"],
        vec!["dev", "cli", "env"],
        vec!["dev", "cli", "contracts"],
        vec!["dev", "cli", "state-audit"],
        vec!["dev", "cli", "state-doctor"],
        vec!["dev", "cli", "runtime-identity"],
        vec!["dev", "cli", "package-health"],
    ];
    for command in commands {
        let mut args = command.to_vec();
        args.push("--format");
        args.push("json");
        args.push("--no-pretty");
        let first = run(&args, &[]);
        let second = run(&args, &[]);
        assert!(first.status.success(), "first run failed for {:?}", args);
        assert!(second.status.success(), "second run failed for {:?}", args);
        assert_eq!(first.stdout, second.stdout, "machine-readable output drift for {:?}", args);
    }
}

#[test]
fn read_only_dev_cli_commands_do_not_mutate_state_files() {
    let root = seed_state_root("side-effects");
    let config = root.join("config.env");
    let history = root.join("history.json");
    let memory = root.join("memory.json");
    let plugins_dir = root.join("plugins");
    let envs = [
        ("BIJUX_CONFIG_PATH", config.to_string_lossy().to_string()),
        ("BIJUX_HISTORY_PATH", history.to_string_lossy().to_string()),
        ("BIJUX_MEMORY_PATH", memory.to_string_lossy().to_string()),
        ("BIJUX_PLUGINS_DIR", plugins_dir.to_string_lossy().to_string()),
    ];
    let before: BTreeMap<&str, Vec<u8>> = BTreeMap::from([
        ("config", file_bytes(&config)),
        ("history", file_bytes(&history)),
        ("memory", file_bytes(&memory)),
    ]);

    let commands = [
        ["dev", "cli", "status"],
        ["dev", "cli", "parity"],
        ["dev", "cli", "contracts"],
        ["dev", "cli", "routes"],
        ["dev", "cli", "registry"],
        ["dev", "cli", "env"],
        ["dev", "cli", "state-audit"],
        ["dev", "cli", "state-doctor"],
        ["dev", "cli", "runtime-identity"],
        ["dev", "cli", "package-health"],
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ];
    for command in commands {
        let out = run(&command, &envs);
        assert!(out.status.success(), "command failed: {:?}", command);
    }

    let after: BTreeMap<&str, Vec<u8>> = BTreeMap::from([
        ("config", file_bytes(&config)),
        ("history", file_bytes(&history)),
        ("memory", file_bytes(&memory)),
    ]);
    assert_eq!(before, after, "read-only dev-cli commands mutated state");
}
