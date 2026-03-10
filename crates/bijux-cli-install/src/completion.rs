#![forbid(unsafe_code)]
//! Shell completion and post-install user guidance.

use std::path::{Path, PathBuf};

/// Shell targets for completion generation during installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionShell {
    /// Bash shell completion.
    Bash,
    /// Zsh shell completion.
    Zsh,
    /// Fish shell completion.
    Fish,
    /// PowerShell completion.
    PowerShell,
}

/// Generate deterministic completion content for an install hook.
#[must_use]
pub fn completion_script(shell: CompletionShell) -> &'static str {
    match shell {
        CompletionShell::Bash => {
            "complete -W \"cli dev doctor version repl completion inspect\" bijux"
        }
        CompletionShell::Zsh => "#compdef bijux\n_arguments '*::command:->commands'",
        CompletionShell::Fish => {
            "complete -c bijux -f -a \"cli dev doctor version repl completion inspect\""
        }
        CompletionShell::PowerShell => {
            "Register-ArgumentCompleter -CommandName bijux -ScriptBlock { param($wordToComplete) }"
        }
    }
}

/// Return the post-install hint shown after successful install.
#[must_use]
pub fn post_install_hint(binary_path: &str) -> String {
    format!(
        "Installed `bijux` at {binary_path}. Run `bijux version` and `bijux cli doctor` to verify your environment."
    )
}

/// Return compatibility note text for cargo users.
#[must_use]
pub fn cargo_compatibility_note() -> &'static str {
    "Cargo installs are canonical for Rust runtime updates. Ensure your PATH resolves to the intended `bijux` binary when pip and cargo are both installed."
}

/// Build completion file path for a shell under a home directory.
#[must_use]
pub fn completion_file_path(shell: CompletionShell, home_dir: &Path) -> PathBuf {
    match shell {
        CompletionShell::Bash => home_dir.join(".bash_completion.d").join("bijux"),
        CompletionShell::Zsh => home_dir.join(".zsh").join("completions").join("_bijux"),
        CompletionShell::Fish => {
            home_dir.join(".config").join("fish").join("completions").join("bijux.fish")
        }
        CompletionShell::PowerShell => {
            home_dir.join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1")
        }
    }
}

/// Detect active shell from explicit value or process environment fallback.
#[must_use]
pub fn detect_shell(shell_env: Option<&str>) -> Option<CompletionShell> {
    let raw = shell_env.map(ToOwned::to_owned).or_else(|| std::env::var("SHELL").ok())?;
    let raw = raw.as_str();
    if raw.contains("bash") {
        return Some(CompletionShell::Bash);
    }
    if raw.contains("zsh") {
        return Some(CompletionShell::Zsh);
    }
    if raw.contains("fish") {
        return Some(CompletionShell::Fish);
    }
    if raw.contains("pwsh") || raw.contains("powershell") {
        return Some(CompletionShell::PowerShell);
    }
    None
}
