#![forbid(unsafe_code)]
//! Replay minimized race reproducers.

use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_bin(args: &[&str]) -> io::Result<Output> {
    for attempt in 0..3 {
        match Command::new(env!("CARGO_BIN_EXE_bijux"))
            .args(args)
            .output()
        {
            Ok(output) => return Ok(output),
            Err(err) if err.kind() == io::ErrorKind::NotFound && attempt < 2 => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => return Err(err),
        }
    }
    unreachable!("retry loop should always return")
}

fn run_case(path: &Path) {
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read case")).expect("parse case");
    let kind = json["kind"].as_str().expect("kind");

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let temp = std::env::temp_dir().join(format!(
        "bijux-race-repro-{}-{}-{nanos}",
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("case"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("mkdir temp");

    let config = temp.join("active.env");
    fs::write(&config, "BIJUXCLI_ALPHA=0\nBIJUXCLI_BETA=0\n").expect("seed config");
    let cfg = Arc::new(config.display().to_string());

    match kind {
        "two-writer-same-key" => {
            let mut jobs = Vec::new();
            for _ in 0..2 {
                let cfg = Arc::clone(&cfg);
                jobs.push(thread::spawn(move || {
                    for _ in 0..40 {
                        let out = run_bin(&[
                            "cli",
                            "config",
                            "set",
                            "alpha=stable",
                            "--config-path",
                            cfg.as_str(),
                        ])
                        .expect("set alpha");
                        assert!(matches!(out.status.code(), Some(0) | Some(1) | Some(2)));
                    }
                }));
            }
            for job in jobs {
                job.join().expect("join same-key writer");
            }
        }
        "read-write-mix" => {
            let writer_cfg = Arc::clone(&cfg);
            let writer = thread::spawn(move || {
                for i in 0..60 {
                    let val = format!("alpha={i}");
                    let out = run_bin(&[
                        "cli",
                        "config",
                        "set",
                        &val,
                        "--config-path",
                        writer_cfg.as_str(),
                    ])
                    .expect("set");
                    assert!(matches!(out.status.code(), Some(0) | Some(1) | Some(2)));
                }
            });
            let reader_cfg = Arc::clone(&cfg);
            let reader = thread::spawn(move || {
                for _ in 0..60 {
                    let out = run_bin(&[
                        "cli",
                        "config",
                        "list",
                        "--format",
                        "json",
                        "--no-pretty",
                        "--config-path",
                        reader_cfg.as_str(),
                    ])
                    .expect("list");
                    assert!(matches!(out.status.code(), Some(0) | Some(1) | Some(2)));
                }
            });
            writer.join().expect("join writer");
            reader.join().expect("join reader");
        }
        _ => panic!("unknown race kind: {kind}"),
    }
}

#[test]
fn minimized_race_reproducers_replay_without_crashing() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fuzz/minimized_cases/state_race_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("read repro directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "at least one minimized race reproducer must be kept"
    );

    for file in files {
        run_case(&file);
    }
}
