use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::{ColorMode, GlobalFlags, LogLevel, OutputFormat, PrettyMode};
use crate::kernel::{build_intent_from_argv, resolve_policy, PolicyInputs};

use super::types::{
    ReplSession, ReplShutdownContract, ReplStartupContract, REPL_PROFILE_MAX_CHARS,
    REPL_PROMPT_MAX_CHARS, REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES,
};

static REPL_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_session_id() -> String {
    let seq = REPL_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis(),
        Err(_) => 0,
    };
    format!("repl-{}-{millis}-{seq}", std::process::id())
}

fn sanitize_text(value: &str, max_chars: usize) -> String {
    value.chars().filter(|ch| !ch.is_control()).take(max_chars).collect()
}

fn normalize_profile(profile: &str) -> String {
    sanitize_text(profile.trim(), REPL_PROFILE_MAX_CHARS)
}

fn default_prompt(profile: &str) -> String {
    if profile.is_empty() {
        "bijux> ".to_string()
    } else {
        format!("bijux[{profile}]> ")
    }
}

fn resolve_prompt(profile: &str, prompt: Option<&str>) -> (String, bool) {
    if let Some(value) = prompt {
        let mut rendered = sanitize_text(value, REPL_PROMPT_MAX_CHARS);
        if rendered.trim().is_empty() {
            rendered = default_prompt(profile);
        } else if !rendered.ends_with(char::is_whitespace) {
            rendered.push(' ');
        }
        let include_profile_context = !profile.is_empty() && rendered.contains(profile);
        return (rendered, include_profile_context);
    }

    (default_prompt(profile), !profile.is_empty())
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

    let profile = normalize_profile(profile);
    let (prompt, include_profile_context) = resolve_prompt(&profile, prompt);
    let session = ReplSession {
        session_id: next_session_id(),
        prompt: prompt.clone(),
        profile,
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
        .map(|namespace| sanitize_text(namespace, REPL_PROFILE_MAX_CHARS))
        .filter(|namespace| !namespace.is_empty())
        .collect::<Vec<_>>();
    namespaces.sort();
    namespaces.dedup();
    let omitted = namespaces.len().saturating_sub(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES);
    namespaces.truncate(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES);
    let mut diagnostics = namespaces
        .into_iter()
        .map(|namespace| format!("plugin {namespace} is broken and will be skipped"))
        .collect::<Vec<_>>();
    if omitted > 0 {
        diagnostics.push(format!("additional broken plugins omitted: {omitted}"));
    }
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

#[cfg(test)]
mod tests {
    use super::{startup_repl, startup_repl_with_diagnostics};
    use crate::interface::repl::types::REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES;

    #[test]
    fn startup_sanitizes_profile_and_prompt_fields() {
        let noisy_profile = "  prod\u{0007}\n";
        let (session, startup) = startup_repl(noisy_profile, Some(" prompt\u{001b}[31m"));
        assert_eq!(session.profile, "prod");
        assert_eq!(startup.prompt, " prompt[31m ");
        assert!(!startup.include_profile_context);
    }

    #[test]
    fn startup_diagnostics_are_sanitized_and_bounded() {
        let mut broken = Vec::new();
        for idx in 0..(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES + 4) {
            broken.push(format!("plugin-{idx:03}\u{0007}"));
        }
        let broken_refs = broken.iter().map(String::as_str).collect::<Vec<_>>();
        let (_session, _startup, diagnostics) =
            startup_repl_with_diagnostics("", None, &broken_refs);
        assert_eq!(diagnostics.len(), REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES + 1);
        assert!(diagnostics.iter().all(|entry| !entry.chars().any(char::is_control)));
        assert!(diagnostics
            .last()
            .is_some_and(|entry| entry.contains("additional broken plugins omitted")));
    }
}
