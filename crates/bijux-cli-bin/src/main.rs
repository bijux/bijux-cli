#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use anyhow::Result;
use bijux_cli_core::core_marker;
use bijux_cli_output::to_json;

fn main() -> Result<()> {
    let marker = core_marker();
    let rendered = to_json(&marker)?;
    println!("{rendered}");
    Ok(())
}
