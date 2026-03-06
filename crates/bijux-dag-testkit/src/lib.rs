//! Shared test helpers for workspace crates.

use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("read json file");
    serde_json::from_str(&text).expect("parse json file")
}
