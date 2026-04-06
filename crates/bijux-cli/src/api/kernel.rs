#![forbid(unsafe_code)]
//! Kernel policy facade.

pub use crate::kernel::{
    map_error_category_to_exit, resolve_policy, ExecutionIntent, PolicyInputs,
};
