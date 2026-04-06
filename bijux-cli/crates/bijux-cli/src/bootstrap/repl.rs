#![forbid(unsafe_code)]
//! Process-backed interactive REPL bootstrap.

use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::Result;

use crate::features::diagnostics::state_paths::resolve_state_paths;
use crate::features::plugins::plugin_doctor;
use crate::interface::cli::parser::parse_intent;
use crate::interface::repl::{
    configure_history, execute_repl_input, flush_history, load_history, shutdown_repl,
    startup_repl, startup_repl_with_diagnostics, ReplEvent, ReplFrame, ReplInput, ReplSession,
    ReplStream,
};
use crate::routing::parser::ParsedGlobalFlags;

fn normalize_line_endings(line: &mut String) {
    while matches!(line.chars().last(), Some('\n' | '\r')) {
        line.pop();
    }
}

fn apply_session_flags(session: &mut ReplSession, flags: &ParsedGlobalFlags) {
    if let Some(format) = flags.output_format {
        session.policy.output_format = format;
    }
    if let Some(mode) = flags.pretty_mode {
        session.policy.pretty_mode = mode;
    }
    if let Some(color) = flags.color_mode {
        session.policy.color_mode = color;
    }
    if let Some(level) = flags.log_level {
        session.policy.log_level = level;
    }
    if flags.quiet {
        session.policy.quiet = true;
        session.policy.log_level = crate::contracts::LogLevel::Error;
    }
    session.config_path.clone_from(&flags.config_path);
}

fn write_frame(frame: ReplFrame) -> io::Result<()> {
    match frame.stream {
        ReplStream::Stdout => {
            let mut stdout = io::stdout().lock();
            write!(stdout, "{}", frame.content)?;
            stdout.flush()
        }
        ReplStream::Stderr => {
            let mut stderr = io::stderr().lock();
            write!(stderr, "{}", frame.content)?;
            stderr.flush()
        }
    }
}

fn write_stderr_line(message: &str) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    writeln!(stderr, "{message}")?;
    stderr.flush()
}

fn initialize_session(flags: &ParsedGlobalFlags) -> Result<ReplSession> {
    let paths = resolve_state_paths(flags)?;
    let doctor = plugin_doctor(&paths.plugin_registry_file);

    let mut session = match &doctor {
        Ok(report) => {
            let broken = report.broken.iter().map(String::as_str).collect::<Vec<_>>();
            let (session, _startup, diagnostics) = startup_repl_with_diagnostics("", None, &broken);
            for diagnostic in diagnostics {
                write_stderr_line(&diagnostic)?;
            }
            session
        }
        Err(error) => {
            write_stderr_line(&format!("plugin registry diagnostics unavailable: {error}"))?;
            let (session, _startup) = startup_repl("", None);
            session
        }
    };

    apply_session_flags(&mut session, flags);
    let history_limit = session.history_limit;
    configure_history(&mut session, Some(paths.history_file), true, history_limit);

    if let Err(error) = load_history(&mut session) {
        write_stderr_line(&format!("history could not be loaded: {error}"))?;
    } else if let Some(message) = session.last_error.clone() {
        write_stderr_line(&message)?;
        session.last_error = None;
    }

    Ok(session)
}

fn run_session_loop(session: &mut ReplSession) -> Result<()> {
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        if interactive {
            let mut stdout = io::stdout().lock();
            write!(stdout, "{}", session.prompt)?;
            stdout.flush()?;
        }

        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        let event = if bytes_read == 0 {
            execute_repl_input(session, ReplInput::Eof)?
        } else {
            normalize_line_endings(&mut line);
            execute_repl_input(session, ReplInput::Line(line.clone()))?
        };

        match event {
            ReplEvent::Continue(Some(frame)) => write_frame(frame)?,
            ReplEvent::Continue(None) => {}
            ReplEvent::Interrupted(frame) => write_frame(frame)?,
            ReplEvent::Exit(Some(frame)) => {
                write_frame(frame)?;
                break;
            }
            ReplEvent::Exit(None) => break,
        }
    }

    Ok(())
}

/// Run the interactive REPL process when argv resolves to the shipped `repl` route.
pub(crate) fn try_run_interactive_repl(argv: &[String]) -> Result<Option<i32>> {
    if argv.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        return Ok(None);
    }

    let intent = match parse_intent(argv) {
        Ok(intent) => intent,
        Err(_) => return Ok(None),
    };

    if !matches!(intent.normalized_path.as_slice(), [a, b] if a == "cli" && b == "repl") {
        return Ok(None);
    }

    let mut session = initialize_session(&intent.global_flags)?;
    run_session_loop(&mut session)?;

    if let Err(error) = flush_history(&session) {
        write_stderr_line(&format!("history could not be saved: {error}"))?;
    }

    let shutdown = shutdown_repl(&session);
    let _ = shutdown;
    Ok(Some(session.last_exit_code))
}
