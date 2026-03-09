#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use anyhow::Result;
use bijux_cli_core::core_marker;
use bijux_cli_output::{EmitterConfig, render_value};
use serde_json::to_value;

fn main() -> Result<()> {
    let marker = core_marker();
    let rendered = render_value(&to_value(marker)?, EmitterConfig::default())?;
    println!("{rendered}");
    Ok(())
}
