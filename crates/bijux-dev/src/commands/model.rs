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

#[cfg(test)]
mod tests {
    use super::{CommandEffect, SuiteSelectionReport};

    #[test]
    fn command_effect_label_is_stable() {
        assert_eq!(CommandEffect::Validation.label(), "validation");
        assert_eq!(CommandEffect::ReadWrite.label(), "read-write");
    }

    #[test]
    fn suite_selection_report_serializes_with_expected_fields() {
        let report = SuiteSelectionReport {
            group: "tests".to_string(),
            selected_suite_ids: vec!["alpha".to_string()],
            skipped_domain: vec![],
            skipped_slow: vec![],
            skipped_internal: vec![],
            skipped_disabled: vec![],
        };
        let value = serde_json::to_value(&report).expect("serialize");
        assert_eq!(value["group"], "tests");
        assert_eq!(value["selected_suite_ids"][0], "alpha");
    }
}
