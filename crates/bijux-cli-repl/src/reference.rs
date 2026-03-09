/// Render stable REPL command reference text.
#[must_use]
pub fn render_repl_command_reference() -> String {
    let lines = [
        "REPL Commands",
        "status",
        "doctor",
        "version",
        ":help <command>",
        ":set trace on|off",
        ":set quiet on|off",
        ":set format json|yaml|text",
        ":plugin reload",
        ":exit",
    ];
    format!("{}\n", lines.join("\n"))
}
