use crate::commands::DagCli;
use std::process::ExitCode;

pub fn dispatch(_cli: DagCli) -> Result<ExitCode, ExitCode> {
    Err(ExitCode::from(3))
}
