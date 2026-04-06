#![forbid(unsafe_code)]
//! Command parser facade.

pub use crate::routing::parser::{
    parse_intent, root_command, ParseError, ParsedGlobalFlags, ParsedIntent,
};
