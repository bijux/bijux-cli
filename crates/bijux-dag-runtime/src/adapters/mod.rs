//! Adapter facade for runtime boundaries.
pub(crate) mod registry;
pub(crate) mod contract;
pub(crate) mod builtins;

pub use crate::adapter_api::*;
pub use crate::adapter_conformance::*;
