#![forbid(unsafe_code)]
//! Ownership boundary contracts for the maintainer command registry and runtime law separation.

use std::collections::BTreeSet;

use bijux_dev_cli::schema::command_registry::{
    command_registry, DevCliCommand, MAINTAINER_COMMAND_NAMESPACE,
};

#[test]
fn command_registry_covers_all_known_maintainer_subcommands() {
    let fixture = include_str!("../data/fixtures/routing/maintainer_subcommands.txt");
    let known: BTreeSet<String> = fixture
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();
    let registered: BTreeSet<String> =
        command_registry().iter().map(|entry| entry.command.as_str().to_string()).collect();

    assert_eq!(registered, known);
}

#[test]
fn command_registry_entries_are_canonical_and_unique() {
    let mut seen = BTreeSet::<&'static str>::new();
    for entry in command_registry() {
        assert_eq!(entry.owner, "bijux-dev-cli");
        assert!(entry.command.as_str().starts_with("bijux-dev-cli "));
        assert!(seen.insert(entry.command.as_str()));
    }
    assert_eq!(MAINTAINER_COMMAND_NAMESPACE, "bijux-dev-cli");
    assert!(command_registry().iter().any(|entry| matches!(entry.command, DevCliCommand::Status)));
}

#[test]
fn crate_scope_rejects_runtime_command_law_and_root_alias_reexports() {
    let lib_source = include_str!("../../../src/maintainer/mod.rs");
    assert!(lib_source.contains("Runtime command law remains in runtime crates"));
    assert!(!lib_source.contains("pub use report"));
    assert!(!lib_source.contains("pub use contract_engine"));

    let runtime_law_signatures = [
        "cli plugins",
        "cli config",
        "history clear",
        "memory set",
        "route_response(",
        "parse_intent(",
    ];

    for signature in runtime_law_signatures {
        let present = include_str!("../../../src/maintainer/reports/control_plane.rs")
            .contains(signature)
            || include_str!("../../../src/maintainer/reports/repository_health/status/mod.rs")
                .contains(signature)
            || include_str!("../../../src/maintainer/reports/runtime_surface/parity.rs")
                .contains(signature)
            || include_str!("../../../src/maintainer/reports/runtime_surface/runtime_identity.rs")
                .contains(signature);
        assert!(
            !present,
            "runtime law signature leaked into maintainer control-plane crate: {signature}"
        );
    }
}
