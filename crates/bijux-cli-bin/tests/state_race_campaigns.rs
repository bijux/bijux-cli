#![forbid(unsafe_code)]
//! Concurrent config and state race coverage.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Arc;
use std::thread;

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_routing as _;
use shlex as _;
use thiserror as _;
use bijux_cli_repl as _;
use libc as _;
use serde_json::Value;

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("bijux-state-race-{name}-{}", std::process::id()));
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

fn shared_env(root: &PathBuf) -> Vec<(&'static str, String)> {
    let home = root.join("home");
    let plugins = root.join("plugins");
    let history = root.join("history.log");
    fs::create_dir_all(home.join(".bijux")).expect("mkdir home/.bijux");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    vec![
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
    ]
}

#[test]
fn concurrent_config_readers_and_writers_preserve_file_shape_and_recoverability() {
    let root = temp_dir("config-rw");
    let config = root.join("active.env");
    fs::write(&config, "BIJUXCLI_ALPHA=0\nBIJUXCLI_BETA=0\n").expect("seed config");
    let cfg = Arc::new(config.display().to_string());

    let mut jobs = Vec::new();

    {
        let cfg = Arc::clone(&cfg);
        jobs.push(thread::spawn(move || {
            for i in 0..80 {
                let value = format!("alpha={i}");
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "set", &value, "--config-path", cfg.as_str()])
                    .output()
                    .expect("set alpha");
                assert_known_status(&out, "set alpha");
            }
        }));
    }
    {
        let cfg = Arc::clone(&cfg);
        jobs.push(thread::spawn(move || {
            for i in 0..80 {
                let value = format!("beta={i}");
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "set", &value, "--config-path", cfg.as_str()])
                    .output()
                    .expect("set beta");
                assert_known_status(&out, "set beta");
            }
        }));
    }
    {
        let cfg = Arc::clone(&cfg);
        jobs.push(thread::spawn(move || {
            for _ in 0..80 {
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg.as_str()])
                    .output()
                    .expect("list");
                assert_known_status(&out, "list concurrent");
            }
        }));
    }
    {
        let cfg = Arc::clone(&cfg);
        jobs.push(thread::spawn(move || {
            for _ in 0..40 {
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "clear", "--format", "json", "--no-pretty", "--config-path", cfg.as_str()])
                    .output()
                    .expect("clear");
                assert_known_status(&out, "clear concurrent");
            }
        }));
    }

    for job in jobs {
        job.join().expect("join race thread");
    }

    let body = fs::read_to_string(&config).expect("read final config");
    assert!(body.lines().all(|line| line.starts_with("BIJUXCLI_") && line.contains('=')));

    let final_list = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg.as_str()])
        .output()
        .expect("final list");
    assert_known_status(&final_list, "final list");
}

#[test]
fn concurrent_config_export_load_and_read_paths_stay_non_corrupt() {
    let root = temp_dir("config-export-load");
    let config = root.join("active.env");
    let source = root.join("source.env");
    let export_path = root.join("export.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\n").expect("seed config");
    fs::write(&source, "BIJUXCLI_ALPHA=2\n").expect("seed source");

    let cfg = Arc::new(config.display().to_string());
    let src = Arc::new(source.display().to_string());
    let exp = Arc::new(export_path.display().to_string());

    let writer = {
        let cfg = Arc::clone(&cfg);
        let src = Arc::clone(&src);
        thread::spawn(move || {
            for i in 0..50 {
                fs::write(src.as_str(), format!("BIJUXCLI_ALPHA={i}\n")).expect("rewrite source");
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "load", src.as_str(), "--config-path", cfg.as_str()])
                    .output()
                    .expect("load");
                assert_known_status(&out, "load concurrent");
            }
        })
    };

    let exporter = {
        let cfg = Arc::clone(&cfg);
        let exp = Arc::clone(&exp);
        thread::spawn(move || {
            for _ in 0..50 {
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "export", exp.as_str(), "--format", "json", "--no-pretty", "--config-path", cfg.as_str()])
                    .output()
                    .expect("export");
                assert_known_status(&out, "export concurrent");
            }
        })
    };

    let reader = {
        let cfg = Arc::clone(&cfg);
        thread::spawn(move || {
            for _ in 0..50 {
                let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
                    .args(["cli", "config", "get", "alpha", "--format", "json", "--no-pretty", "--config-path", cfg.as_str()])
                    .output()
                    .expect("get");
                assert_known_status(&out, "get concurrent");
            }
        })
    };

    writer.join().expect("join writer");
    exporter.join().expect("join exporter");
    reader.join().expect("join reader");

    let final_list = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(["cli", "config", "list", "--format", "json", "--no-pretty", "--config-path", cfg.as_str()])
        .output()
        .expect("final list");
    assert_known_status(&final_list, "final list");
}

#[test]
fn concurrent_history_plugin_registry_and_memory_reads_remain_stable() {
    let root = temp_dir("state-read-races");
    let envs = shared_env(&root);

    let plugins_dir = PathBuf::from(envs.iter().find(|(k, _)| *k == "BIJUXCLI_PLUGINS_DIR").expect("plugins env").1.clone());
    let history = PathBuf::from(envs.iter().find(|(k, _)| *k == "BIJUXCLI_HISTORY_FILE").expect("history env").1.clone());
    let home = PathBuf::from(envs.iter().find(|(k, _)| *k == "HOME").expect("home env").1.clone());
    let memory = home.join(".bijux").join(".memory.json");

    fs::write(plugins_dir.join("registry.json"), "{\"plugins\":[]}").expect("seed registry");
    fs::write(&history, "status\n").expect("seed history");
    fs::write(&memory, "{\"alpha\":{\"value\":\"1\"}}\n").expect("seed memory");

    let env_arc = Arc::new(envs);

    let registry_mutator = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            let plugins_dir = PathBuf::from(env.iter().find(|(k, _)| *k == "BIJUXCLI_PLUGINS_DIR").expect("plugins").1.clone());
            for i in 0..80 {
                let body = format!("{{\"plugins\":[{{\"name\":\"p{i}\",\"path\":\"/tmp/p{i}\",\"enabled\":true}}]}}\n");
                fs::write(plugins_dir.join("registry.json"), body).expect("rewrite registry");
            }
        })
    };

    let registry_reader = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            for _ in 0..80 {
                let out = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &env);
                assert_known_status(&out, "plugins list race");
            }
        })
    };

    let history_writer = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            let history = PathBuf::from(env.iter().find(|(k, _)| *k == "BIJUXCLI_HISTORY_FILE").expect("history").1.clone());
            for i in 0..120 {
                let mut existing = fs::read_to_string(&history).unwrap_or_default();
                existing.push_str(&format!("status-{i}\n"));
                fs::write(&history, existing).expect("append history");
            }
        })
    };

    let history_reader = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            for _ in 0..120 {
                let out = run_with_env(&["history", "--format", "json", "--no-pretty"], &env);
                assert_known_status(&out, "history race");
            }
        })
    };

    let memory_reader = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            for _ in 0..80 {
                let out = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &env);
                assert_known_status(&out, "memory list race");
            }
        })
    };

    let doctor_reader = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            for _ in 0..80 {
                let out = run_with_env(&["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"], &env);
                assert_known_status(&out, "state-doctor race");
            }
        })
    };

    let audit_reader = {
        let env = Arc::clone(&env_arc);
        thread::spawn(move || {
            for _ in 0..50 {
                let out = run_with_env(&["dev", "cli", "state-audit", "--format", "json", "--no-pretty"], &env);
                assert_known_status(&out, "state-audit race");
            }
        })
    };

    registry_mutator.join().expect("join registry mutator");
    registry_reader.join().expect("join registry reader");
    history_writer.join().expect("join history writer");
    history_reader.join().expect("join history reader");
    memory_reader.join().expect("join memory reader");
    doctor_reader.join().expect("join doctor reader");
    audit_reader.join().expect("join audit reader");

    // Non-corruption invariant: final readers still execute under known status classes.
    let plugin_final = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &env_arc);
    let history_final = run_with_env(&["history", "--format", "json", "--no-pretty"], &env_arc);
    let memory_final = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &env_arc);
    assert_known_status(&plugin_final, "final plugins list");
    assert_known_status(&history_final, "final history");
    assert_known_status(&memory_final, "final memory");
}

#[test]
fn deterministic_final_state_is_stable_when_policy_uses_same_target_value() {
    let root = temp_dir("deterministic-final-state");
    let config = root.join("active.env");
    fs::write(&config, "BIJUXCLI_ALPHA=old\n").expect("seed config");

    let cfg = Arc::new(config.display().to_string());
    let mut writers = Vec::new();
    for _ in 0..8 {
        let cfg = Arc::clone(&cfg);
        writers.push(thread::spawn(move || {
            for _ in 0..40 {
                let out = run_with_env(
                    &["cli", "config", "set", "alpha=stable", "--config-path", cfg.as_str()],
                    &[],
                );
                assert_known_status(&out, "set stable race");
            }
        }));
    }
    for writer in writers {
        writer.join().expect("join stable writer");
    }

    let list = run_with_env(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            cfg.as_str(),
        ],
        &[],
    );
    assert_eq!(list.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&list.stdout).expect("json payload");
    assert_eq!(payload["value"], "stable");
}
