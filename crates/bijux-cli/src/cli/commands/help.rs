//! Backward-compatible shim for CLI help rendering.

#[allow(dead_code)]
pub(crate) fn render_command_help(path: &[&str]) -> anyhow::Result<String> {
    crate::interface::cli::help::render_command_help(path)
}
