use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::{ColorMode, GlobalFlags, LogLevel, OutputFormat, PrettyMode};
use crate::kernel::{build_intent_from_argv, resolve_policy, PolicyInputs};

use super::types::{ReplSession, ReplShutdownContract, ReplStartupContract};

static REPL_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_session_id() -> String {
    let seq = REPL_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    format!("repl-{}-{millis}-{seq}", std::process::id())
}

fn resolve_prompt(profile: &str, prompt: Option<&str>) -> (String, bool) {
    if let Some(value) = prompt {
        let rendered = value.to_string();
        let include_profile_context = !profile.trim().is_empty() && rendered.contains(profile);
        return (rendered, include_profile_context);
    }

    if profile.trim().is_empty() {
        return ("bijux> ".to_string(), false);
    }

    (format!("bijux[{profile}]> "), true)
}

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

    let (prompt, include_profile_context) = resolve_prompt(profile, prompt);
    let session = ReplSession {
        session_id: next_session_id(),
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
        completion_registries: BTreeMap::new(),
    };

    let startup = ReplStartupContract { prompt, include_profile_context, policy };

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
    let mut namespaces = broken_plugins
        .iter()
        .map(|namespace| namespace.trim())
        .filter(|namespace| !namespace.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    namespaces.sort();
    namespaces.dedup();
    let diagnostics = namespaces
        .into_iter()
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
