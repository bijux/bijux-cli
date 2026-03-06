use bijux_dag_app::{dag_command, dag_run};
use clap::Command;
use std::process::ExitCode;

fn main() -> ExitCode {
    let matches = Command::new("bijux")
        .about("Bijux umbrella CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(dag_command())
        .subcommand(Command::new("rag").about("Not implemented"))
        .subcommand(Command::new("rar").about("Not implemented"))
        .get_matches();

    match matches.subcommand() {
        Some(("dag", sub)) => match dag_run(sub) {
            Ok(code) => code,
            Err(code) => code,
        },
        Some(("rag", _)) | Some(("rar", _)) => {
            eprintln!("not implemented");
            ExitCode::from(2)
        }
        _ => ExitCode::from(2),
    }
}
