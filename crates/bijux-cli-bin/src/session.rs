use std::collections::BTreeMap;

use bijux_cli_core::kernel::{build_intent_from_argv, resolve_policy, PolicyInputs};
use bijux_cli_routing::{ColorMode, GlobalFlags, LogLevel, OutputFormat, PrettyMode};

use crate::types::{ReplSession, ReplShutdownContract, ReplStartupContract};

/// Startup REPL session using the same policy precedence and routing registry as CLI.
#[must_use]
pub fn startup_repl(profile: &str, prompt: Option<&str>) -> (ReplSession, ReplStartupContract) {
    let defaults = GlobalFlags {
        output_format: Some(OutputFormat::Json),
        pretty_mode: Some(PrettyMode::Pretty),
        color_mode: Some(ColorMode::Never),
        log_level: Some(LogLevel::Info),
        quiet: false,
        include_runtime: false,
    };

    let policy = resolve_policy(
        &build_intent_from_argv(&["bijux".to_string(), "repl".to_string()]),
        &PolicyInputs { env: defaults.clone(), config: defaults.clone(), defaults },
    );

    let prompt = prompt.unwrap_or("bijux> ").to_string();
    let session = ReplSession {
        session_id: "repl-1".to_string(),
        prompt: prompt.clone(),
        profile: profile.to_string(),
        policy: policy.clone(),
        commands_executed: 0,
        last_exit_code: 0,
        trace_mode: false,
        history: Vec::new(),
        history_limit: 500,
        history_enabled: true,
        history_file: None,
        pending_multiline: None,
        last_error: None,
        plugin_completion_hooks: BTreeMap::new(),
    };

    let startup =
        ReplStartupContract { prompt, include_profile_context: !profile.is_empty(), policy };

    (session, startup)
}

/// Startup REPL with startup diagnostics for preflight issues.
#[must_use]
pub fn startup_repl_with_diagnostics(
    profile: &str,
    prompt: Option<&str>,
    broken_plugins: &[&str],
) -> (ReplSession, ReplStartupContract, Vec<String>) {
    let (session, startup) = startup_repl(profile, prompt);
    let diagnostics = broken_plugins
        .iter()
        .map(|namespace| format!("plugin {namespace} is broken and will be skipped"))
        .collect();
    (session, startup, diagnostics)
}

/// Shutdown REPL session and emit stable contract.
#[must_use]
pub fn shutdown_repl(session: &ReplSession) -> ReplShutdownContract {
    ReplShutdownContract {
        session_id: session.session_id.clone(),
        commands_executed: session.commands_executed,
    }
}
