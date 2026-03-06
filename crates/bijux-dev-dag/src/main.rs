use clap::{Parser, Subcommand};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

const FORBIDDEN_DEPS: &[(&str, &str)] = &[
    ("crates/bijux-dag-runtime/Cargo.toml", "bijux-dag-app"),
    ("crates/bijux-dag-core/Cargo.toml", "bijux-dag-runtime"),
    ("crates/bijux-dag-runtime/Cargo.toml", "bijux-dag-cli"),
    ("crates/bijux-dag-core/Cargo.toml", "bijux-dag-artifacts"),
];

#[derive(Parser)]
#[command(name = "bijux-dev-dag")]
#[command(about = "Developer workflow helpers for bijux-dag")]
struct Cli {
    #[command(subcommand)]
    command: CommandLine,
}

#[derive(Subcommand)]
enum CommandLine {
    /// Run cargo fmt on workspace
    Fmt,
    /// Run workspace format check + clippy
    Lint,
    /// Run cargo audit
    Security,
    /// Run metadata + tests + format check
    Sanity,
    /// Generate and verify golden run/replay contract
    Golden,
    /// Compare cargo-public-api output with docs/api baseline
    PublicApi,
    /// Check forbidden dependency usage in workspace Cargo manifests
    DepGuard,
    /// Run full CI-like sequence
    Ci,
    /// Run CLI compatibility command
    Compat,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run(command: CommandLine) -> Result<(), String> {
    match command {
        CommandLine::Fmt => run_status("cargo", &["fmt", "--all"]),
        CommandLine::Lint => {
            run_status("cargo", &["fmt", "--all", "--", "--check"])?;
            run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        }
        CommandLine::Security => run_status("cargo", &["audit"]),
        CommandLine::Sanity => {
            run_status("cargo", &["metadata", "--no-deps"])?;
            run_status("cargo", &["test", "-q"])?;
            run_status("cargo", &["fmt", "--all", "--", "--check"])
        }
        CommandLine::Golden => run_golden(),
        CommandLine::PublicApi => run_public_api(),
        CommandLine::DepGuard => run_dep_guard(),
        CommandLine::Ci => run_ci(),
        CommandLine::Compat => run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"]),
    }
}

fn run_ci() -> Result<(), String> {
    run_status("cargo", &["fmt", "--all"])?;
    run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    run_dep_guard()?;
    run_status("cargo", &["test", "--workspace"])?;
    run_golden()?;
    run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"])?;

    let root = repo_root()?;
    let scratch = std::env::temp_dir().join(format!("bijux-dag-ci-{}", now_secs()));
    let runs = scratch.join("runs");
    fs::create_dir_all(&runs).map_err(|err| err.to_string())?;
    run_with_root(
        &root,
        "cargo",
        &["run", "-p", "bijux-dag-cli", "--", "dag", "run", "examples/hello.dag.json", "--out", runs.to_str().expect("utf-8")],
    )?;
    let run_dir = newest_run(&runs)?;
    run_status_in_dir(
        &root,
        "cargo",
        &["run", "-p", "bijux-dag-cli", "--", "dag", "verify", run_dir.to_str().expect("utf-8")],
    )
}

fn run_golden() -> Result<(), String> {
    let root = repo_root()?;
    let scratch = std::env::temp_dir().join(format!("bijux-dag-golden-{}", now_secs()));
    let runs = scratch.join("runs");
    fs::create_dir_all(&runs).map_err(|err| err.to_string())?;

    let example = "examples/hello.dag.json";
    for _ in 0..2 {
        run_with_root(
            &root,
            "cargo",
            &["run", "-p", "bijux-dag-cli", "--", "dag", "run", example, "--out", runs.to_str().expect("utf-8")],
        )?;
    }

    let (latest, previous) = two_latest_runs(&runs)?;

    let diff = run_status_and_json(
        &root,
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "diff",
            previous.to_str().expect("utf-8"),
            latest.to_str().expect("utf-8"),
            "--json",
        ],
    )?;
    assert_empty_diff(&diff)?;

    run_with_root(
        &root,
        "cargo",
        &["run", "-p", "bijux-dag-cli", "--", "dag", "replay", latest.to_str().expect("utf-8"), "--out", runs.to_str().expect("utf-8")],
    )?;

    let replay = newest_run(&runs)?;
    let replay_diff = run_status_and_json(
        &root,
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "diff",
            latest.to_str().expect("utf-8"),
            replay.to_str().expect("utf-8"),
            "--json",
        ],
    )?;
    assert_empty_diff(&replay_diff)
}

fn run_public_api() -> Result<(), String> {
    if Command::new("cargo-public-api").arg("--version").status().is_err() {
        return Ok(());
    }
    let root = repo_root()?;
    let docs_api = root.join("docs/api");
    fs::create_dir_all(&docs_api).map_err(|err| err.to_string())?;

    for crate_name in ["bijux-dag-core", "bijux-dag-artifacts", "bijux-dag-runtime", "bijux-dag-app"] {
        let output = run_stdout_and_json(
            &root,
            "cargo",
            &["public-api", "-p", crate_name],
        )?;
        let out_txt = docs_api.join(format!("{crate_name}.txt"));
        if out_txt.exists() {
            let baseline = fs::read_to_string(&out_txt).map_err(|err| err.to_string())?;
            if baseline != output {
                return Err(format!("public API changed for {crate_name}"));
            }
        } else {
            fs::write(&out_txt, output).map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

fn run_dep_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut failed = false;
    for (manifest, dep) in FORBIDDEN_DEPS {
        if manifest_forbidden(&root.join(manifest), dep)? {
            eprintln!("forbidden dependency: {dep} in {manifest}");
            failed = true;
        }
    }
    if failed {
        Err("dependency guard failed".into())
    } else {
        Ok(())
    }
}

fn manifest_forbidden(path: &Path, dependency: &str) -> Result<bool, String> {
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let quoted = format!("\"{dependency}\"");
    for line in content.lines() {
        let t = line.trim_start();
        if t.starts_with(&format!("{dependency} =")) || t.contains(&quoted) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn assert_empty_diff(diff: &Value) -> Result<(), String> {
    if diff.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!("expected ok=true: {diff}"));
    }
    let payload = diff
        .get("data")
        .ok_or_else(|| "missing data field".to_string())?;
    let is_empty_object = |key: &str| {
        payload
            .get(key)
            .map(|v| v.is_object() && v.as_object().is_some_and(|m| m.is_empty()))
            .unwrap_or(false)
    };

    if !is_empty_object("manifest") {
        return Err(format!("manifest not empty: {payload}"));
    }
    if payload
        .get("graph_fingerprint")
        .and_then(Value::as_null)
        .is_none()
    {
        return Err(format!("graph_fingerprint not null: {payload}"));
    }
    if !is_empty_object("nodes") {
        return Err(format!("nodes not empty: {payload}"));
    }
    if !is_empty_object("outputs") {
        return Err(format!("outputs not empty: {payload}"));
    }
    Ok(())
}

fn run_status(cmd: &str, args: &[&str]) -> Result<(), String> {
    run_status_in_dir(&repo_root()?, cmd, args)
}

fn run_status_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|err| format!("failed to run {cmd}: {err}"))?;
    if !status.success() {
        return Err(format!("`{cmd}` failed with status {status}"));
    }
    Ok(())
}

fn run_with_root(root: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    run_status_in_dir(root, cmd, args)
}

fn run_status_and_json(root: &Path, args: &[&str]) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("failed to run cargo: {err}"))?;
    if !output.status.success() {
        let _ = io::stdout().write_all(&output.stdout);
        let _ = io::stderr().write_all(&output.stderr);
        return Err(format!("cargo failed with status {}", output.status));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).map_err(|err| format!("invalid json: {err}\nstdout:\n{stdout}"))
}

fn run_stdout_and_json(root: &Path, cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("failed to run {cmd}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{cmd} failed with status {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn newest_run(runs: &Path) -> Result<PathBuf, String> {
    let mut candidates: Vec<_> = fs::read_dir(runs)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();

    candidates.sort_by(|a, b| {
        let ma = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| format!("no runs found in {}", runs.display()))
}

fn two_latest_runs(runs: &Path) -> Result<(PathBuf, PathBuf), String> {
    let mut candidates: Vec<_> = fs::read_dir(runs)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|n| n.starts_with("run-"))
        })
        .collect();

    if candidates.len() < 2 {
        return Err(format!("expected at least 2 runs in {}", runs.display()));
    }

    candidates.sort_by(|a, b| {
        let ma = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });

    Ok((candidates[0].clone(), candidates[1].clone()))
}

fn repo_root() -> Result<PathBuf, String> {
    let mut dir = env::current_dir().map_err(|err| err.to_string())?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("could not locate repo root".to_string())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
