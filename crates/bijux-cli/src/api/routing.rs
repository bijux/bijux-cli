#![forbid(unsafe_code)]
//! Route identity and registry facade.

/// Canonical route catalog helpers.
pub mod catalog {
    pub use crate::routing::catalog::{
        cli_config_subcommands, cli_plugins_subcommands, cli_root_aliases, is_known_route,
        normalize_command_path, repl_reference_commands,
    };
}

/// Parser and normalization interfaces.
pub mod parser {
    pub use crate::routing::parser::{
        parse_intent, root_command, ParseError, ParsedGlobalFlags, ParsedIntent,
    };
}

/// Registry resolution interfaces.
pub mod registry {
    pub use crate::routing::registry::{RouteError, RouteRegistry, RouteTarget};
}
