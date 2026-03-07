use serde::Serialize;
use std::path::PathBuf;

#[derive(Copy, Clone)]
pub(crate) enum CommandEffect {
    Validation,
    ReadWrite,
}

impl CommandEffect {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::ReadWrite => "read-write",
        }
    }
}

pub(crate) struct SuiteDef {
    pub(crate) id: &'static str,
    pub(crate) description: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) slow: bool,
    pub(crate) internal: bool,
    pub(crate) effect: CommandEffect,
    pub(crate) run: fn() -> Result<(), String>,
}

pub(crate) struct CommandContext {
    pub(crate) json: bool,
    pub(crate) report: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SuiteSelectionReport {
    pub(crate) group: String,
    pub(crate) selected_suite_ids: Vec<String>,
    pub(crate) skipped_domain: Vec<String>,
    pub(crate) skipped_slow: Vec<String>,
    pub(crate) skipped_internal: Vec<String>,
    pub(crate) skipped_disabled: Vec<String>,
}
