#![forbid(unsafe_code)]
//! CLI parser, dispatch, and help surfaces.

pub mod contracts;
pub mod dispatch;
pub(crate) mod handlers;
pub mod help;
pub mod parser;
