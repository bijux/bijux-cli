use clap::{Parser, Subcommand};
use serde_json::json;
use serde_json::Value;
use sha2::Digest;
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
    #[arg(long)]
    json: bool,
    #[arg(long)]
    report: Option<PathBuf>,
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
    /// Run legacy style checks
    Checks {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Run legacy style tests
    Tests {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Run compatibility and runtime contracts
    Contracts {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Produce documentation health or report views
    Docs {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Run release preparation workflows
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    /// Run repo and governance policies
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    /// Validate and preview scheduling definitions
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommand,
    },
    /// Print environment diagnostics and report status
    Doctor,
    /// Generate and verify golden run/replay contract
    Golden,
    /// Compare cargo-public-api output with docs/api baseline
    PublicApi,
    /// API surface commands
    Api {
        #[command(subcommand)]
        command: ApiCommand,
    },
    /// Check forbidden dependency usage in workspace Cargo manifests
    DepGuard,
    /// Remove workspace target artifacts
    ArtifactsClean,
    /// Print build environment summary
    EnvSummary,
    /// Validate required cargo tools are installed
    VerifyTools,
    /// Verify workspace dependencies resolve
    ResolveCheck,
    /// Record baseline benchmark artifact
    BenchmarkBaseline,
    /// Record memory smoke artifact
    MemorySmoke,
    /// Verify artifact reproducibility and integrity for local runs
    ArtifactVerify,
    /// Run full CI-like sequence
    Ci,
    /// Run CLI compatibility command
    Compat,
}

#[derive(Subcommand)]
enum ControlCommand {
    /// Execute suite checks
    Run {
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        include_slow: bool,
        #[arg(long)]
        include_internal: bool,
    },
    /// Show known suites
    List,
    /// Explain a suite
    Explain {
        #[arg(long)]
        suite: String,
    },
}

#[derive(Subcommand)]
enum RepoCommand {
    /// Run dependency policy checks
    Deps,
    /// Execute governance suites
    Run {
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        include_slow: bool,
        #[arg(long)]
        include_internal: bool,
    },
    /// Show known repo suites
    List,
    /// Explain a repo suite
    Explain {
        #[arg(long)]
        suite: String,
    },
}

#[derive(Subcommand)]
enum ScheduleCommand {
    /// Validate schedule registry semantics
    Validate {
        #[arg(long, default_value = "configs/schedules/registry.json")]
        file: PathBuf,
    },
    /// Preview next-fire behavior
    Preview {
        #[arg(long, default_value = "configs/schedules/registry.json")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum ApiCommand {
    /// Verify public API surface contracts
    PublicSurface,
}

#[derive(Subcommand)]
enum ReleaseCommand {
    /// Execute release verification
    Verify,
    /// List release workflows
    List,
    /// Explain a release workflow
    Explain {
        #[arg(long)]
        suite: String,
    },
}

#[derive(Copy, Clone)]
enum CommandEffect {
    Validation,
    ReadWrite,
}

impl CommandEffect {
    fn label(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::ReadWrite => "read-write",
        }
    }
}

struct SuiteDef {
    id: &'static str,
    description: &'static str,
    domain: &'static str,
    slow: bool,
    internal: bool,
    effect: CommandEffect,
    run: fn() -> Result<(), String>,
}

const CHECK_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "fmt",
        description: "cargo fmt check",
        domain: "style",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["fmt", "--all", "--", "--check"]),
    },
    SuiteDef {
        id: "lint",
        description: "cargo clippy with warnings as errors",
        domain: "quality",
        slow: true,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_status("cargo", &["fmt", "--all", "--", "--check"])?;
            run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        },
    },
    SuiteDef {
        id: "security",
        description: "cargo audit policy check",
        domain: "supply-chain",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["audit"]),
    },
    SuiteDef {
        id: "dep-guard",
        description: "forbidden dependency reference check",
        domain: "policy",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_dep_guard(),
    },
];

const TEST_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "unit",
        description: "cargo test --workspace",
        domain: "runtime",
        slow: true,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "--workspace"]),
    },
    SuiteDef {
        id: "arch",
        description: "repository architecture tests",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "-p", "bijux-dev-dag"]),
    },
];

const CONTRACT_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "compat",
        description: "core compat fixture assertions",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"]),
    },
    SuiteDef {
        id: "golden",
        description: "run/replay golden execution parity",
        domain: "runtime",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_golden(),
    },
    SuiteDef {
        id: "public-api",
        description: "public API surface contract",
        domain: "quality",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_public_api(),
    },
];

const DOC_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "api",
        description: "check documentation index files",
        domain: "docs",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            let root = repo_root()?;
            if !root.join("docs").join("DEVELOPMENT.md").exists() {
                return Err("missing docs/DEVELOPMENT.md".into());
            }
            Ok(())
        },
    },
];

const RELEASE_SUITES: &[SuiteDef] = &[SuiteDef {
    id: "verify",
    description: "full release verification",
    domain: "release",
    slow: true,
    internal: false,
    effect: CommandEffect::ReadWrite,
    run: || run_ci(),
}];

const REPO_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "dependency-policy",
        description: "legacy workspace dependency reference check",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_missing_workspace_dependency_checks(),
    },
    SuiteDef {
        id: "dep-guard",
        description: "forbidden crate import check",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_dep_guard(),
    },
];

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

struct CommandContext {
    json: bool,
    report: Option<PathBuf>,
}

fn run(cli: Cli) -> Result<(), String> {
    let context = CommandContext {
        json: cli.json,
        report: cli.report,
    };
    match cli.command {
        CommandLine::Fmt => run_command_reported(&context, "fmt", CommandEffect::Validation, json!({}), || {
            run_status("cargo", &["fmt", "--all"])
        }),
        CommandLine::Lint => run_command_reported(&context, "lint", CommandEffect::Validation, json!({}), || {
            run_status("cargo", &["fmt", "--all", "--", "--check"])?;
            run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        }),
        CommandLine::Security => run_command_reported(&context, "security", CommandEffect::Validation, json!({}), || {
            run_status("cargo", &["audit"])
        }),
        CommandLine::Sanity => run_command_reported(&context, "sanity", CommandEffect::ReadWrite, json!({}), || {
            run_status("cargo", &["metadata", "--no-deps"])?;
            run_status("cargo", &["test", "-q"])?;
            run_status("cargo", &["fmt", "--all", "--", "--check"])
        }),
        CommandLine::Checks { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "checks", CHECK_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            ControlCommand::List => {
                run_suite_list(&context, "checks", CHECK_SUITES)
            }
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "checks", &suite, CHECK_SUITES)
            }
        },
        CommandLine::Tests { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "tests", TEST_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            ControlCommand::List => {
                run_suite_list(&context, "tests", TEST_SUITES)
            }
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "tests", &suite, TEST_SUITES)
            }
        },
        CommandLine::Contracts { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(
                    &context,
                    "contracts",
                    CONTRACT_SUITES,
                    &domain,
                    fail_fast,
                    include_slow,
                    include_internal,
                )
            }
            ControlCommand::List => {
                run_suite_list(&context, "contracts", CONTRACT_SUITES)
            }
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "contracts", &suite, CONTRACT_SUITES)
            }
        },
        CommandLine::Docs { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "docs", DOC_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            ControlCommand::List => {
                run_suite_list(&context, "docs", DOC_SUITES)
            }
            ControlCommand::Explain { suite } => run_suite_explain(&context, "docs", &suite, DOC_SUITES),
        },
        CommandLine::Release { command } => match command {
            ReleaseCommand::Verify => {
                run_command_reported(&context, "release.verify", CommandEffect::ReadWrite, json!({}), || run_ci())
            }
            ReleaseCommand::List => {
                run_suite_list(&context, "release", RELEASE_SUITES)
            }
            ReleaseCommand::Explain { suite } => {
                run_suite_explain(&context, "release", &suite, RELEASE_SUITES)
            }
        },
        CommandLine::Repo { command } => match command {
            RepoCommand::Deps => {
                run_command_reported(&context, "repo.deps", CommandEffect::Validation, json!({}), || {
                    run_missing_workspace_dependency_checks()
                })
            }
            RepoCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "repo", REPO_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            RepoCommand::List => run_suite_list(&context, "repo", REPO_SUITES),
            RepoCommand::Explain { suite } => run_suite_explain(&context, "repo", &suite, REPO_SUITES),
        },
        CommandLine::Schedule { command } => match command {
            ScheduleCommand::Validate { file } => run_command_reported(
                &context,
                "schedule.validate",
                CommandEffect::Validation,
                json!({ "file": file }),
                || run_schedule_validate(&file),
            ),
            ScheduleCommand::Preview { file } => run_command_reported(
                &context,
                "schedule.preview",
                CommandEffect::Validation,
                json!({ "file": file }),
                || run_schedule_preview(&file),
            ),
        },
        CommandLine::Doctor => run_command_reported(&context, "doctor", CommandEffect::ReadWrite, json!({}), || {
            run_env_summary()?;
            run_verify_tools()
        }),
        CommandLine::Golden => run_command_reported(&context, "golden", CommandEffect::ReadWrite, json!({}), || {
            run_golden()
        }),
        CommandLine::PublicApi => run_command_reported(&context, "public-api", CommandEffect::ReadWrite, json!({}), || {
            run_public_api()
        }),
        CommandLine::DepGuard => run_command_reported(&context, "dep-guard", CommandEffect::Validation, json!({}), || {
            run_dep_guard()
        }),
        CommandLine::ArtifactsClean => run_command_reported(&context, "artifacts-clean", CommandEffect::ReadWrite, json!({}), || {
            run_artifacts_clean()
        }),
        CommandLine::EnvSummary => run_command_reported(&context, "env-summary", CommandEffect::Validation, json!({}), || {
            run_env_summary()
        }),
        CommandLine::VerifyTools => run_command_reported(&context, "verify-tools", CommandEffect::Validation, json!({}), || {
            run_verify_tools()
        }),
        CommandLine::ResolveCheck => run_command_reported(&context, "resolve-check", CommandEffect::Validation, json!({}), || {
            run_resolve_check()
        }),
        CommandLine::BenchmarkBaseline => run_command_reported(
            &context,
            "benchmark-baseline",
            CommandEffect::ReadWrite,
            json!({}),
            || run_benchmark_baseline(),
        ),
        CommandLine::MemorySmoke => run_command_reported(
            &context,
            "memory-smoke",
            CommandEffect::ReadWrite,
            json!({}),
            || run_memory_smoke(),
        ),
        CommandLine::ArtifactVerify => run_command_reported(
            &context,
            "artifact-verify",
            CommandEffect::Validation,
            json!({}),
            || run_artifact_verify(),
        ),
        CommandLine::Ci => run_command_reported(&context, "ci", CommandEffect::ReadWrite, json!({}), || {
            run_ci()
        }),
        CommandLine::Compat => run_command_reported(&context, "compat", CommandEffect::ReadWrite, json!({}), || {
            run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"])
        }),
        CommandLine::Api { command } => match command {
            ApiCommand::PublicSurface => run_command_reported(&context, "api.public-surface", CommandEffect::ReadWrite, json!({}), || {
                run_public_api()
            }),
        },
    }
}

fn run_suite_group(
    context: &CommandContext,
    group: &str,
    suites: &[SuiteDef],
    domain: &Option<String>,
    fail_fast: bool,
    include_slow: bool,
    include_internal: bool,
) -> Result<(), String> {
    let selected: Vec<&SuiteDef> = suites
        .iter()
        .filter(|suite| domain.as_deref().is_none_or(|d| suite.domain == d))
        .filter(|suite| include_internal || !suite.internal)
        .filter(|suite| include_slow || !suite.slow)
        .collect();

    let mut failed: Vec<String> = Vec::new();
    for suite in selected {
        if let Err(error) = run_suite(context, group, suite) {
            failed.push(format!("{}: {error}", suite.id));
            if fail_fast {
                break;
            }
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{} failed: {}", group, failed.join(", ")))
    }
}

fn run_suite(context: &CommandContext, group: &str, suite: &SuiteDef) -> Result<(), String> {
    run_command_reported(context, &format!("{group}.{}", suite.id), suite.effect, json!({}), suite.run)
}

fn run_suite_list(context: &CommandContext, group: &str, suites: &[SuiteDef]) -> Result<(), String> {
    let data = json!({
        "group": group,
        "suites": suites.iter().map(|s| json!({"id": s.id, "description": s.description, "domain": s.domain, "slow": s.slow, "internal": s.internal, "effect": s.effect.label()})).collect::<Vec<_>>()
    });
    run_text_or_json_report(
        context,
        group,
        &format!("{group}.list"),
        "read-write",
        data,
        || Ok(()),
        false,
    )
}

fn run_suite_explain(context: &CommandContext, group: &str, suite_id: &str, suites: &[SuiteDef]) -> Result<(), String> {
    let suite = suites
        .iter()
        .find(|suite| suite.id == suite_id)
        .ok_or_else(|| format!("suite '{suite_id}' is unknown"))?;
    let data = json!({
        "id": suite.id,
        "group": group,
        "description": suite.description,
        "domain": suite.domain,
        "slow": suite.slow,
        "internal": suite.internal,
        "effect": suite.effect.label(),
    });
    run_text_or_json_report(
        context,
        group,
        &format!("{group}.explain"),
        suite.effect.label(),
        data,
        || Ok(()),
        false,
    )
}

fn run_command_reported<F>(context: &CommandContext, command: &str, effect: CommandEffect, data: Value, run: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    run_text_or_json_report(context, command, command, effect.label(), data, run, true)
}

fn run_text_or_json_report(
    context: &CommandContext,
    command: &str,
    command_name: &str,
    effect: &str,
    data: Value,
    run: impl FnOnce() -> Result<(), String>,
    include_data_on_success: bool,
) -> Result<(), String> {
    let result = run();
    let (status, error) = match &result {
        Ok(_) => ("ok", None),
        Err(err) => ("error", Some(err.clone())),
    };

    let mut report = json!({
        "command": command_name,
        "status": status,
        "effect": effect,
        "data": data,
    });
    if let Some(error) = error {
        report["error"] = Value::String(error);
    }

    if context.json {
        println!("{}", serde_json::to_string_pretty(&report).expect("json print"));
    } else if include_data_on_success || status == "error" {
        let value = report.to_string();
        println!("[{command}] {status} ({effect}): {value}",);
    } else {
        println!("[{command}] {status} ({effect})");
    }

    if let Some(report_path) = context.report.as_ref() {
        let output = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
        fs::write(report_path, output).map_err(|err| err.to_string())?;
    }

    result
}

fn run_ci() -> Result<(), String> {
    run_status("cargo", &["fmt", "--all"])?;
    run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    run_dep_guard()?;
    run_resolve_check()?;
    run_missing_workspace_dependency_checks()?;
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

fn run_schedule_validate(file: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(file);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read schedule file {}: {err}", path.display()))?;
    let payload: Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse schedule file {}: {err}", path.display()))?;
    let definitions = payload
        .get("definitions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "schedule registry must contain a 'definitions' array".to_string())?;

    let mut seen = std::collections::BTreeSet::new();
    for definition in definitions {
        let id = definition
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "schedule definition is missing string 'id'".to_string())?;
        if !seen.insert(id.to_string()) {
            return Err(format!("duplicate schedule id '{id}'"));
        }
        let trigger = definition
            .get("trigger")
            .ok_or_else(|| format!("schedule '{id}' is missing 'trigger'"))?;
        let trigger_kind = trigger
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("schedule '{id}' trigger is missing 'kind'"))?;
        if trigger_kind == "cron" {
            let expression = trigger
                .get("expression")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("schedule '{id}' cron trigger is missing 'expression'"))?;
            let parts: Vec<&str> = expression.split_whitespace().collect();
            if parts.len() != 5 {
                return Err(format!(
                    "schedule '{id}' cron expression must have exactly five fields"
                ));
            }
        }
    }
    Ok(())
}

fn run_schedule_preview(file: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(file);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read schedule file {}: {err}", path.display()))?;
    let payload: Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse schedule file {}: {err}", path.display()))?;
    let definitions = payload
        .get("definitions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "schedule registry must contain a 'definitions' array".to_string())?;
    let now = now_millis();
    for definition in definitions {
        let id = definition
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let trigger = definition.get("trigger").unwrap_or(&Value::Null);
        let kind = trigger
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let preview = if kind == "cron" { now + 60_000 } else { now };
        println!("schedule={id} trigger={kind} preview_unix_ms={preview}");
    }
    Ok(())
}

fn run_artifacts_clean() -> Result<(), String> {
    let root = repo_root()?;
    let artifacts_target = root.join("artifacts").join("target");
    if !artifacts_target.exists() {
        println!("artifacts target path is already clean: {}", artifacts_target.display());
        return Ok(());
    }
    fs::remove_dir_all(&artifacts_target).map_err(|err| err.to_string())?;
    println!("removed artifacts target: {}", artifacts_target.display());
    Ok(())
}

fn run_env_summary() -> Result<(), String> {
    println!("repo_root={}", repo_root()?.display());
    println!("cwd={}", env::current_dir().map_err(|err| err.to_string())?.display());
    print_command_version("rustc");
    print_command_version("cargo");
    print_command_version("cargo-audit");
    print_command_version("cargo-public-api");
    print_command_version("cargo-nextest");
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        println!("CARGO_TARGET_DIR={target_dir}");
    } else {
        println!("CARGO_TARGET_DIR=<not_set>");
    }
    Ok(())
}

fn print_command_version(command: &str) {
    let output = Command::new(command)
        .arg("--version")
        .output()
        .ok();
    if let Some(output) = output {
        if output.status.success() {
            println!(
                "{}={}",
                command,
                String::from_utf8_lossy(&output.stdout).trim()
            );
        } else {
            println!("{}=<unavailable>", command);
        }
    } else {
        println!("{}=<unavailable>", command);
    }
}

fn run_verify_tools() -> Result<(), String> {
    let mut failed = false;
    for tool in ["cargo-audit", "cargo-public-api", "cargo-nextest", "rustup"] {
        let status = Command::new(tool).arg("--version").status();
        match status {
            Ok(status) if status.success() => println!("tool available: {tool}"),
            Ok(_) => {
                failed = true;
                println!("tool failed to execute: {tool}");
            }
            Err(err) => {
                failed = true;
                println!("tool missing: {tool} ({err})");
            }
        }
    }
    if failed {
        Err("required tools are missing or unavailable".into())
    } else {
        Ok(())
    }
}

fn run_resolve_check() -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|err| format!("cargo metadata failed: {err}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed with status {}", output.status));
    }
    let payload = String::from_utf8_lossy(&output.stdout);
    if payload.contains("\"packages\"") {
        println!("workspace metadata resolved");
        Ok(())
    } else {
        Err("cargo metadata output missing package list".into())
    }
}

fn run_benchmark_baseline() -> Result<(), String> {
    let root = repo_root()?;
    let out_dir = root.join("artifacts").join("benchmarks");
    let runs_dir = out_dir.join("runs");
    fs::create_dir_all(&runs_dir).map_err(|err| err.to_string())?;

    let start_ms = now_millis();
    run_with_root(
        &root,
        "cargo",
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "run",
            "benchmarks/fixtures/large_dag.json",
            "--out",
            runs_dir.to_str().ok_or_else(|| "non-utf8 runs path".to_string())?,
        ],
    )?;
    let end_ms = now_millis();
    let report = json!({
        "fixture": "benchmarks/fixtures/large_dag.json",
        "elapsed_ms": end_ms.saturating_sub(start_ms),
        "recorded_at_unix_ms": end_ms
    });
    fs::write(
        out_dir.join("baseline.json"),
        serde_json::to_vec_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn run_memory_smoke() -> Result<(), String> {
    let root = repo_root()?;
    let out_dir = root.join("artifacts").join("memory");
    let runs_dir = out_dir.join("runs");
    fs::create_dir_all(&runs_dir).map_err(|err| err.to_string())?;

    let start_ms = now_millis();
    run_with_root(
        &root,
        "cargo",
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "run",
            "examples/hello.dag.json",
            "--out",
            runs_dir.to_str().ok_or_else(|| "non-utf8 runs path".to_string())?,
        ],
    )?;
    let end_ms = now_millis();
    let report = json!({
        "workload": "examples/hello.dag.json",
        "elapsed_ms": end_ms.saturating_sub(start_ms),
        "memory_budget_note": "Track peak memory through CI runner metrics and fail on sustained regressions.",
        "recorded_at_unix_ms": end_ms
    });
    fs::write(
        out_dir.join("smoke.json"),
        serde_json::to_vec_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn run_artifact_verify() -> Result<(), String> {
    let root = repo_root()?;
    let runs_root = root.join("artifacts").join("runs");
    if !runs_root.exists() {
        println!("no artifact runs directory found at {}", runs_root.display());
        return Ok(());
    }

    let mut failures = Vec::new();
    for entry in fs::read_dir(&runs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let run_path = entry.path();
        if !run_path.is_dir() {
            continue;
        }
        let name = run_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !name.starts_with("run-") {
            continue;
        }
        let manifest_path = run_path.join("manifest.json");
        if !manifest_path.exists() {
            failures.push(format!("{name}: missing manifest.json"));
            continue;
        }
        let manifest_text = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
        let manifest: serde_json::Value =
            serde_json::from_str(&manifest_text).map_err(|err| err.to_string())?;
        let outputs = manifest
            .get("outputs")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for output in outputs {
            let node_id = output
                .get("node_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let file = output
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let expected_sha = output
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let file_path = run_path.join("nodes").join(node_id).join("outputs").join(file);
            if !file_path.exists() {
                failures.push(format!("{name}: missing output {}", file_path.display()));
                continue;
            }
            let bytes = fs::read(&file_path).map_err(|err| err.to_string())?;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&bytes);
            let actual_sha = hex::encode(hasher.finalize());
            if actual_sha != expected_sha {
                failures.push(format!(
                    "{name}: sha mismatch for {}",
                    file_path.display()
                ));
            }
        }
    }

    if failures.is_empty() {
        println!("artifact verification passed");
        Ok(())
    } else {
        Err(format!("artifact verification failed: {}", failures.join(", ")))
    }
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

fn run_missing_workspace_dependency_checks() -> Result<(), String> {
    let root = repo_root()?;
    let manifests = [
        "crates/bijux-dag-core/Cargo.toml",
        "crates/bijux-dag-artifacts/Cargo.toml",
        "crates/bijux-dag-runtime/Cargo.toml",
        "crates/bijux-dag-app/Cargo.toml",
        "crates/bijux-dag-cli/Cargo.toml",
        "crates/bijux-dev-dag/Cargo.toml",
    ];
    let mut failed = false;
    for manifest in manifests {
        let content = fs::read_to_string(root.join(manifest)).map_err(|err| err.to_string())?;
        for line in content.lines() {
            if line.contains("bijux_dag_") {
                eprintln!("legacy workspace crate reference in {manifest}: {line}");
                failed = true;
            }
        }
    }
    if failed {
        Err("found legacy workspace dependency references".into())
    } else {
        println!("workspace dependency references use canonical names");
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

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
