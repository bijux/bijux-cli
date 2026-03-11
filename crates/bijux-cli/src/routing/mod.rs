#![forbid(unsafe_code)]
//! Routing graph, parser normalization, and namespace resolution.

pub mod catalog;
pub(crate) mod model;
pub mod parser;
pub mod registry;

#[cfg(test)]
use proptest as _;

#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_json as _;
