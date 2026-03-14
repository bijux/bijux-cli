#![forbid(unsafe_code)]
//! Shell completion and post-install user guidance.

use std::path::{Path, PathBuf};

use crate::routing::registry::RouteRegistry;

/// Shell targets for completion generation during installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionShell {
    /// Bash shell completion.
    Bash,
    /// Zsh shell completion.
    Zsh,
    /// Fish shell completion.
    Fish,
    /// `pwsh` completion on supported POSIX hosts.
    PowerShell,
}

impl CompletionShell {
    /// Parse a shell selector from CLI input.
    #[must_use]
    pub fn from_cli_value(raw: &str) -> Option<Self> {
        match raw {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "pwsh" => Some(Self::PowerShell),
            _ => None,
        }
    }
}

/// Generate deterministic completion content for an install hook.
#[must_use]
#[allow(dead_code)]
pub(crate) fn completion_script(shell: CompletionShell) -> String {
    let commands = RouteRegistry::default()
        .render_command_tree()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    match shell {
        CompletionShell::Bash => format!("complete -W \"{commands}\" bijux"),
        CompletionShell::Zsh => format!("#compdef bijux\n_arguments '*::command:({commands})'"),
        CompletionShell::Fish => format!("complete -c bijux -f -a \"{commands}\""),
        CompletionShell::PowerShell => format!(
            "Register-ArgumentCompleter -CommandName bijux -ScriptBlock {{ param($wordToComplete) \"{commands}\".Split(' ') | Where-Object {{ $_ -like \"$wordToComplete*\" }} }}"
        ),
    }
}

/// Return the post-install hint shown after successful install.
#[must_use]
pub fn post_install_hint(binary_path: &str) -> String {
    format!(
        "Installed `bijux` at {binary_path}. Run `bijux version` and `bijux doctor` to verify your environment."
    )
}

/// Build completion file path for a shell under a home directory.
#[must_use]
#[allow(dead_code)]
pub(crate) fn completion_file_path(shell: CompletionShell, home_dir: &Path) -> PathBuf {
    match shell {
        CompletionShell::Bash => home_dir.join(".bash_completion.d").join("bijux"),
        CompletionShell::Zsh => home_dir.join(".zsh").join("completions").join("_bijux"),
        CompletionShell::Fish => {
            home_dir.join(".config").join("fish").join("completions").join("bijux.fish")
        }
        CompletionShell::PowerShell => {
            home_dir.join(".config").join("powershell").join("Microsoft.PowerShell_profile.ps1")
        }
    }
}

/// Detect active shell from explicit value or process environment fallback.
#[must_use]
#[allow(dead_code)]
pub(crate) fn detect_shell(shell_env: Option<&str>) -> Option<CompletionShell> {
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
    if raw.contains("pwsh") {
        return Some(CompletionShell::PowerShell);
    }
    None
}
