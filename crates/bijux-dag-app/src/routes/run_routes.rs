use crate::commands::{CacheModeArg, DagCli, MaterializeModeArg};
use crate::routes::preconditions::require_file;
use crate::run_data::map_materialize_mode;
use crate::{emit_json, parse_graph, parse_selectors, read_file, ExitCode};
use bijux_dag_runtime::{CacheMode, Runtime, RuntimeConfig};
use serde_json::json;
use std::path::{Path, PathBuf};

pub(crate) struct RunRouteRequest<'a> {
    pub dag: &'a Path,
    pub out: &'a Path,
    pub run_id: Option<String>,
    pub latest: Option<PathBuf>,
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub node_timeout_ms: Option<u64>,
    pub run_timeout_ms: Option<u64>,
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    pub hermetic: bool,
    pub select: &'a Vec<String>,
    pub exclude: &'a Vec<String>,
    pub materialize_inputs: MaterializeModeArg,
    pub cache: CacheModeArg,
    pub cache_dir: Option<PathBuf>,
    pub remote_cache_dir: Option<PathBuf>,
}

pub(crate) fn handle_run_command(
    cli: &DagCli,
    req: RunRouteRequest<'_>,
) -> Result<ExitCode, ExitCode> {
    require_file(req.dag)?;
    let input = read_file(req.dag)?;
    let graph = parse_graph(&input)?;
    let runtime = Runtime::new();
    let (deny_network, deny_clock, clean_env) = effective_policy_flags(
        req.deny_network,
        req.deny_clock,
        req.clean_env,
        req.hermetic,
    );
    let deny_env = req.deny_env;
    let selectors = parse_selectors(req.select, req.exclude)?;
    let options = RuntimeConfig {
        jobs: req.jobs,
        cpu_budget: req.cpu_budget,
        run_timeout_ms: req.run_timeout_ms,
        node_timeout_ms: req.node_timeout_ms,
        materialize_inputs: map_materialize_mode(req.materialize_inputs),
        cache_mode: match req.cache {
            CacheModeArg::Off => CacheMode::Off,
            CacheModeArg::Read => CacheMode::Read,
            CacheModeArg::Readwrite => CacheMode::ReadWrite,
        },
        cache_dir: req.cache_dir,
        remote_cache_dir: req.remote_cache_dir,
        run_id: req.run_id,
        latest_symlink: req.latest,
        policy: bijux_dag_runtime::PolicyConfig {
            deny_network,
            deny_env,
            deny_clock,
            clean_env,
        },
        selectors,
        ..RuntimeConfig::default()
    };
    let run_path = runtime
        .run(&graph, req.out, options)
        .map_err(|_| ExitCode::from(3))?;

    if cli.json {
        return emit_json(
            cli,
            "dag.run",
            true,
            json!({"run_dir": run_path}),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    if !cli.quiet {
        println!("run dir: {}", run_path.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn effective_policy_flags(
    deny_network: bool,
    deny_clock: bool,
    clean_env: bool,
    hermetic: bool,
) -> (bool, bool, bool) {
    if hermetic {
        return (true, true, true);
    }
    let _ = clean_env;
    let normalized_clean_env = true;
    (deny_network, deny_clock, normalized_clean_env)
}

#[cfg(test)]
mod tests {
    use super::effective_policy_flags;

    #[test]
    fn hermetic_forces_isolation_flags() {
        assert_eq!(
            effective_policy_flags(false, false, false, true),
            (true, true, true)
        );
    }

    #[test]
    fn non_hermetic_preserves_network_clock_and_normalizes_clean_env() {
        assert_eq!(
            effective_policy_flags(true, false, false, false),
            (true, false, true)
        );
    }
}
