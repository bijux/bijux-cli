#![forbid(unsafe_code)]
//! Shared time helpers for deterministic reporting surfaces.

use std::time::{SystemTime, UNIX_EPOCH};

/// Current unix timestamp in seconds.
#[must_use]
pub fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
