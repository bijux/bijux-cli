#![forbid(unsafe_code)]
//! Python bridge execution parity coverage for stable behavior contracts.

use bijux_cli::api::runtime::run_app;
use bijux_cli_python::{
    classify_failure, execution_facade_api, execution_outcome_api, BridgeErrorKind,
};
use serde_json::Value;

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("valid json")
}

#[test]
fn python_bridge_version_status_doctor_and_inspect_match_binary_outputs() {
    let commands = [
        vec!["bijux", "version"],
        vec!["bijux", "status"],
        vec!["bijux", "doctor"],
        vec!["bijux", "inspect", "--format", "json", "--no-pretty"],
    ];

    for command in commands {
        let argv: Vec<String> = command.into_iter().map(ToString::to_string).collect();
        let bridge = execution_facade_api(&argv).expect("bridge execution");
        let core = run_app(&argv).expect("binary execution");
        assert_eq!(bridge, core.stdout, "bridge output mismatch for {:?}", argv);
    }
}

#[test]
fn python_bridge_plugins_config_history_and_memory_match_binary_outputs() {
    let commands = [
        vec!["bijux", "cli", "plugins", "list"],
        vec!["bijux", "cli", "config", "get", "bridge_execution_probe_key"],
        vec!["bijux", "history", "--format", "json", "--no-pretty"],
        vec!["bijux", "memory", "--format", "json", "--no-pretty"],
    ];

    for command in commands {
        let argv: Vec<String> = command.into_iter().map(ToString::to_string).collect();
        let bridge_outcome = parse_json(&execution_outcome_api(&argv).expect("bridge outcome"));
        let core = run_app(&argv).expect("binary execution");
        assert_eq!(
            bridge_outcome["exit_code"].as_i64().unwrap_or(-1),
            i64::from(core.exit_code),
            "exit mismatch for {:?}",
            argv
        );
        assert_eq!(
            bridge_outcome["stdout"].as_str().unwrap_or_default(),
            core.stdout,
            "stdout mismatch for {:?}",
            argv
        );
        assert_eq!(
            bridge_outcome["stderr"].as_str().unwrap_or_default(),
            core.stderr,
            "stderr mismatch for {:?}",
            argv
        );
    }
}

#[test]
fn python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives(
) {
    let representative_failures = [
        ("usage", vec!["bijux", "ghost", "status"]),
        (
            "validation",
            vec![
                "bijux",
                "cli",
                "plugins",
                "scaffold",
                "python",
                "cli",
                "--path",
                "/tmp/bijux-bridge-validation",
            ],
        ),
        ("plugin", vec!["bijux", "cli", "plugins", "inspect", "community-missing"]),
        (
            "internal",
            vec!["bijux", "cli", "config", "load", "/tmp/bijux-bridge-missing-config.env"],
        ),
    ];

    for (label, command) in representative_failures {
        let argv: Vec<String> = command.into_iter().map(ToString::to_string).collect();
        let bridge = parse_json(&execution_outcome_api(&argv).expect("bridge outcome"));
        let core = run_app(&argv).expect("binary execution");
        assert_eq!(
            bridge["exit_code"].as_i64().unwrap_or(-1),
            i64::from(core.exit_code),
            "exit mismatch for representative failure class {label}"
        );
    }
}

#[test]
fn python_bridge_and_binary_agree_on_stream_routing_for_covered_commands() {
    let commands = [vec!["bijux", "status"], vec!["bijux", "ghost", "status"]];

    for command in commands {
        let argv: Vec<String> = command.into_iter().map(ToString::to_string).collect();
        let bridge = parse_json(&execution_outcome_api(&argv).expect("bridge outcome"));
        let core = run_app(&argv).expect("binary execution");
        assert_eq!(
            bridge["stdout"].as_str().unwrap_or_default(),
            core.stdout,
            "stdout mismatch for {:?}",
            argv
        );
        assert_eq!(
            bridge["stderr"].as_str().unwrap_or_default(),
            core.stderr,
            "stderr mismatch for {:?}",
            argv
        );
    }
}

#[test]
fn python_bridge_and_binary_agree_on_namespace_rejection_behavior() {
    let argv = vec!["bijux".to_string(), "ghost".to_string(), "status".to_string()];
    let bridge = parse_json(&execution_outcome_api(&argv).expect("bridge outcome"));
    let core = run_app(&argv).expect("binary execution");

    assert_eq!(bridge["exit_code"].as_i64().unwrap_or(-1), i64::from(core.exit_code));
    assert_eq!(
        bridge["error_kind"].as_str().unwrap_or_default(),
        match classify_failure(core.exit_code, &core.stderr) {
            BridgeErrorKind::Usage => "UsageError",
            BridgeErrorKind::Validation => "ValidationError",
            BridgeErrorKind::Internal => "InternalError",
        }
    );
}

#[test]
fn python_bridge_and_binary_help_outputs_match_for_representative_commands() {
    let commands =
        [vec!["bijux", "status", "--help"], vec!["bijux", "cli", "plugins", "list", "--help"]];

    for command in commands {
        let argv: Vec<String> = command.into_iter().map(ToString::to_string).collect();
        let bridge = execution_facade_api(&argv).expect("bridge execution");
        let core = run_app(&argv).expect("binary execution");
        let expected = if core.exit_code == 0 {
            core.stdout
        } else if !core.stderr.is_empty() {
            core.stderr
        } else {
            core.stdout
        };
        assert_eq!(bridge, expected, "help mismatch for {:?}", argv);
    }
}
