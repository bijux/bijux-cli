#![forbid(unsafe_code)]
//! Canonical maintainer command registry and ownership boundary contracts.

use std::collections::BTreeSet;

use bijux_dev_cli::{command_registry, DevCliCommand, MAINTAINER_COMMAND_NAMESPACE};

#[test]
fn command_registry_covers_all_known_dev_cli_subcommands() {
    let fixture =
        include_str!("../../bijux-cli/tests/data/fixtures/routing/dev_cli_subcommands.txt");
    let known: BTreeSet<String> = fixture
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();
    let registered: BTreeSet<String> =
        command_registry().iter().map(|entry| entry.command.as_str().to_string()).collect();

    assert_eq!(
        registered, known,
        "every known dev cli subcommand must be owned by bijux-dev-cli registry"
    );
}

#[test]
fn command_registry_entries_are_canonical_and_unique() {
    let mut seen = BTreeSet::<&'static str>::new();
    for entry in command_registry() {
        assert_eq!(entry.owner, "bijux-dev-cli");
        assert!(entry.command.as_str().starts_with("dev cli "));
        assert!(
            seen.insert(entry.command.as_str()),
            "duplicate registry entry: {}",
            entry.command.as_str()
        );
    }
    assert_eq!(MAINTAINER_COMMAND_NAMESPACE, "dev cli");
}

#[test]
fn dev_cli_crate_does_not_define_runtime_command_law() {
    let lib_source = include_str!("../src/lib.rs");
    assert!(
        lib_source.contains("Runtime command law remains in runtime crates"),
        "crate-level scope must explicitly reject runtime command-law ownership"
    );

    let runtime_law_signatures = [
        "cli plugins",
        "cli config",
        "history clear",
        "memory set",
        "route_response(",
        "parse_intent(",
    ];

    for signature in runtime_law_signatures {
        let present = include_str!("../src/commands/control_plane.rs").contains(signature)
            || include_str!("../src/commands/status/mod.rs").contains(signature)
            || include_str!("../src/commands/parity.rs").contains(signature)
            || include_str!("../src/commands/runtime_identity.rs").contains(signature);
        assert!(
            !present,
            "dev cli crate must not define runtime command law signature: {signature}"
        );
    }
}

#[test]
fn command_registry_includes_internal_and_visible_commands() {
    let has_visible = command_registry().iter().any(|entry| entry.visible);
    let has_internal = command_registry().iter().any(|entry| !entry.visible);

    assert!(has_visible, "registry must include visible maintainer commands");
    assert!(has_internal, "registry must include internal maintainer commands");

    assert!(
        command_registry().iter().any(|entry| matches!(entry.command, DevCliCommand::Status)),
        "registry must include status command"
    );
}
