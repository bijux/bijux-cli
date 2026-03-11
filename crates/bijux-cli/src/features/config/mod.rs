#![forbid(unsafe_code)]
//! Configuration feature internals.

pub(crate) mod error;
pub(crate) mod operations;
pub(crate) mod serialization;
pub(crate) mod service;
/// Config persistence repository interfaces used by maintainer query adapters.
pub mod storage;
pub(crate) mod validation;
