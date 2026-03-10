use bijux_cli_routing::catalog::repl_reference_commands;

/// Render stable REPL command reference text.
#[must_use]
pub fn render_repl_command_reference() -> String {
    let mut lines = vec!["REPL Commands"];
    lines.extend(repl_reference_commands());
    format!("{}\n", lines.join("\n"))
}
