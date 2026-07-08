use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct MaintainerCommandSurfaceContract {
    schema_version: String,
    owner: String,
    binary: String,
    visible_root_commands: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_contract() -> MaintainerCommandSurfaceContract {
    let path = repo_root().join("contracts/foundation/maintainer_command_surface.v1.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("invalid {}: {err}", path.display()))
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

fn maintainer_root_help() -> String {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-dev", "--bin", "bijux-dev-dag", "--", "--help"])
        .current_dir(repo_root())
        .output()
        .expect("run bijux-dev-dag --help");
    assert!(output.status.success(), "bijux-dev-dag --help failed");
    String::from_utf8(output.stdout).expect("help output must be utf8")
}

fn parse_root_help_commands(help: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut in_commands = false;

    for line in help.lines() {
        if line == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if line == "Options:" {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        commands.push(trimmed.split_whitespace().next().expect("command token").to_string());
    }

    commands
}

#[test]
fn maintainer_command_surface_contract_is_current() {
    let contract = read_contract();
    assert_eq!(contract.schema_version, "foundation-maintainer-command-surface/v1");
    assert_eq!(contract.owner, "bijux-dev");
    assert_eq!(contract.binary, "bijux-dev-dag");
    assert!(!contract.visible_root_commands.is_empty(), "visible root commands must not be empty");
}

#[test]
fn maintainer_root_help_matches_governed_command_surface() {
    let contract = read_contract();
    let help = maintainer_root_help();
    let actual = parse_root_help_commands(&help);

    assert_eq!(
        actual, contract.visible_root_commands,
        "visible bijux-dev-dag --help command inventory drifted from the governed maintainer surface"
    );
}

#[test]
fn maintainer_command_surface_docs_and_policy_inventory_stay_linked() {
    let command_surface = read_repo_file("docs/bijux-dev/operations/command-surface.md");
    let package_doc = read_repo_file("docs/bijux-dev/packages/bijux-dev.md");
    let root_policy_report =
        read_repo_file("docs/bijux-core/foundation/root-policy-surface-report.md");
    let root_policy_inventory =
        read_repo_file("contracts/foundation/root_policy_surface_inventory.v1.json");
    let release_operations = read_repo_file("docs/bijux-dev/operations/release-operations.md");
    let test_policy = read_repo_file("docs/bijux-dev/governance/test-policy.md");

    for content in [
        &command_surface,
        &package_doc,
        &root_policy_report,
        &root_policy_inventory,
        &release_operations,
        &test_policy,
    ] {
        assert!(
            content.contains("contracts/foundation/maintainer_command_surface.v1.json"),
            "maintainer command-surface contract must stay linked from docs and policy inventory"
        );
    }

    for required in
        ["`bijux-dev-dag` Root Surface", "`repo`", "`release`", "`verify`", "`dag`", "`foundation`"]
    {
        assert!(
            command_surface.contains(required),
            "docs/bijux-dev/operations/command-surface.md must document {required}"
        );
    }

    for required in [
        "`bijux-dev-cli docs write-dag-cli-reference`",
        "`bijux-dev-cli maintenance ignored-dag-tests`",
    ] {
        assert!(
            command_surface.contains(required)
                || release_operations.contains(required)
                || test_policy.contains(required),
            "maintainer operations docs must mention {required}"
        );
    }
}
