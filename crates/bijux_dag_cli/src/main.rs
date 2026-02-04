use bijux_dag_app::{dag_command, dag_run};
use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!("warning: bijux-dag is deprecated; use `bijux dag ...`");
    let matches = dag_command().get_matches();
    match dag_run(&matches) {
        Ok(code) => code,
        Err(code) => code,
    }
}
