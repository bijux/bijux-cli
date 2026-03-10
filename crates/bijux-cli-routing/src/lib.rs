#![forbid(unsafe_code)]
//! Routing graph and namespace resolution surfaces.

pub mod catalog;
pub mod parser;
pub mod reports;
pub mod registry;

#[cfg(test)]
use proptest as _;

#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_json as _;
