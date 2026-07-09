use crate::commands::model::{CommandContext, SuiteDef, SuiteSelectionReport};
use crate::commands::reporting::{run_command_reported, run_text_or_json_report};
use serde_json::json;
use std::collections::BTreeSet;

pub(crate) fn run_suite_group(
    context: &CommandContext,
    group: &str,
    suites: &[SuiteDef],
    domain: &Option<String>,
    fail_fast: bool,
    include_slow: bool,
    include_internal: bool,
    advisory: bool,
    why: bool,
) -> Result<(), String> {
    let root = crate::commands::repo_root()?;
    let overrides =
        crate::suites::load_suite_overrides(&root.join("configs/dag/dev/suite_overrides.json"))?;
    let disabled: BTreeSet<String> = overrides.disabled_suite_ids.into_iter().collect();
    let selection =
        build_suite_selection(group, suites, domain, include_slow, include_internal, &disabled);

    if why {
        let details = serde_json::to_value(&selection).map_err(|err| err.to_string())?;
        run_text_or_json_report(
            context,
            group,
            &format!("{group}.why"),
            "validation",
            details,
            || Ok(()),
            true,
        )?;
    }

    let selected: Vec<&SuiteDef> = suites
        .iter()
        .filter(|suite| selection.selected_suite_ids.iter().any(|id| id == suite.id))
        .collect();

    let mut failed: Vec<String> = Vec::new();
    for suite in selected {
        if let Err(error) = run_suite(context, group, suite) {
            failed.push(format!("{}: {error}", suite.id));
            if fail_fast {
                break;
            }
        }
    }
    finalize_suite_outcome(group, failed, advisory, context)
}

fn finalize_suite_outcome(
    group: &str,
    failed: Vec<String>,
    advisory: bool,
    context: &CommandContext,
) -> Result<(), String> {
    if failed.is_empty() || advisory {
        if advisory && !failed.is_empty() {
            run_text_or_json_report(
                context,
                group,
                &format!("{group}.advisory-summary"),
                "validation",
                json!({ "status": "advisory", "failed": failed }),
                || Ok(()),
                true,
            )?;
        }
        Ok(())
    } else {
        Err(format!("{} failed: {}", group, failed.join(", ")))
    }
}

pub(crate) fn run_suite_list(
    context: &CommandContext,
    group: &str,
    suites: &[SuiteDef],
) -> Result<(), String> {
    let data = json!({
        "group": group,
        "suites": suites.iter().map(|s| json!({"id": s.id, "description": s.description, "domain": s.domain, "slow": s.slow, "internal": s.internal, "effect": s.effect.label()})).collect::<Vec<_>>()
    });
    run_text_or_json_report(
        context,
        group,
        &format!("{group}.list"),
        "read-write",
        data,
        || Ok(()),
        false,
    )
}

pub(crate) fn run_suite_explain(
    context: &CommandContext,
    group: &str,
    suite_id: &str,
    suites: &[SuiteDef],
) -> Result<(), String> {
    let suite = suites
        .iter()
        .find(|suite| suite.id == suite_id)
        .ok_or_else(|| format!("suite '{suite_id}' is unknown"))?;
    let data = json!({
        "id": suite.id,
        "group": group,
        "description": suite.description,
        "domain": suite.domain,
        "slow": suite.slow,
        "internal": suite.internal,
        "effect": suite.effect.label(),
    });
    run_text_or_json_report(
        context,
        group,
        &format!("{group}.explain"),
        suite.effect.label(),
        data,
        || Ok(()),
        false,
    )
}

fn run_suite(context: &CommandContext, group: &str, suite: &SuiteDef) -> Result<(), String> {
    run_command_reported(
        context,
        &format!("{group}.{}", suite.id),
        suite.effect,
        json!({}),
        suite.run,
    )
}

fn build_suite_selection(
    group: &str,
    suites: &[SuiteDef],
    domain: &Option<String>,
    include_slow: bool,
    include_internal: bool,
    disabled: &BTreeSet<String>,
) -> SuiteSelectionReport {
    let mut selected_suite_ids = Vec::new();
    let mut skipped_domain = Vec::new();
    let mut skipped_slow = Vec::new();
    let mut skipped_internal = Vec::new();
    let mut skipped_disabled = Vec::new();
    for suite in suites {
        if domain.as_deref().is_some_and(|d| suite.domain != d) {
            skipped_domain.push(suite.id.to_string());
            continue;
        }
        if !include_internal && suite.internal {
            skipped_internal.push(suite.id.to_string());
            continue;
        }
        if !include_slow && suite.slow {
            skipped_slow.push(suite.id.to_string());
            continue;
        }
        if disabled.contains(suite.id) {
            skipped_disabled.push(suite.id.to_string());
            continue;
        }
        selected_suite_ids.push(suite.id.to_string());
    }
    SuiteSelectionReport {
        group: group.to_string(),
        selected_suite_ids,
        skipped_domain,
        skipped_slow,
        skipped_internal,
        skipped_disabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::model::CommandEffect;

    fn pass() -> Result<(), String> {
        Ok(())
    }

    #[test]
    fn selection_reports_reasons_by_filter_type() {
        let suites = vec![
            SuiteDef {
                id: "a",
                description: "a",
                domain: "repo",
                slow: false,
                internal: false,
                effect: CommandEffect::Validation,
                run: pass,
            },
            SuiteDef {
                id: "b",
                description: "b",
                domain: "docs",
                slow: true,
                internal: false,
                effect: CommandEffect::Validation,
                run: pass,
            },
            SuiteDef {
                id: "c",
                description: "c",
                domain: "repo",
                slow: false,
                internal: true,
                effect: CommandEffect::Validation,
                run: pass,
            },
        ];
        let mut disabled = BTreeSet::new();
        disabled.insert("a".to_string());
        let selection = build_suite_selection(
            "checks",
            &suites,
            &Some("repo".to_string()),
            false,
            false,
            &disabled,
        );
        assert!(selection.selected_suite_ids.is_empty());
        assert_eq!(selection.skipped_domain, vec!["b".to_string()]);
        assert_eq!(selection.skipped_internal, vec!["c".to_string()]);
        assert_eq!(selection.skipped_disabled, vec!["a".to_string()]);
    }

    #[test]
    fn blocking_mode_returns_error_on_failures() {
        let context = CommandContext { json: false, report: None };
        let result =
            finalize_suite_outcome("checks", vec!["lint: failed".to_string()], false, &context);
        assert!(result.is_err());
    }

    #[test]
    fn advisory_mode_does_not_fail_command() {
        let context = CommandContext { json: true, report: None };
        let result =
            finalize_suite_outcome("checks", vec!["lint: failed".to_string()], true, &context);
        assert!(result.is_ok());
    }
}
