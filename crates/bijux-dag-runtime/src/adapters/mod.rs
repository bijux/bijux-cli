//! Adapter facade for runtime boundaries.
pub(crate) mod builtins;
pub(crate) mod contract;
pub(crate) mod registry;

pub use crate::adapter_api::*;
pub use crate::adapter_conformance::*;
