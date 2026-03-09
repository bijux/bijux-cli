#![forbid(unsafe_code)]
//! Shell completion and post-install user guidance.

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

/// Return compatibility note text for pip users.
#[must_use]
pub fn pip_compatibility_note() -> &'static str {
    "Pip installs should expose the same `bijux` binary behavior as cargo builds. Run `bijux version` and `bijux cli doctor` after upgrades."
}

/// Return compatibility note text for cargo users.
#[must_use]
pub fn cargo_compatibility_note() -> &'static str {
    "Cargo installs are canonical for Rust runtime updates. Ensure your PATH resolves to the intended `bijux` binary when pip and cargo are both installed."
}
