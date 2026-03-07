use bijux_dag_app::{dag_command, dag_run};
use clap::{Arg, Command as Cmd};
use clap_complete::{generate, shells};
#[cfg(test)]
use serde_json as _;
use std::panic::AssertUnwindSafe;
use std::process::ExitCode;
#[cfg(test)]
use tempfile as _;

fn main() -> ExitCode {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 2 && argv.get(1).is_some_and(|arg| arg == "dag") {
        let mut dag = dag_command();
        let _ = dag.print_help();
        println!();
        return ExitCode::SUCCESS;
    }

    let mut cmd = Cmd::new("bijux")
        .about("Bijux umbrella CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(dag_command())
        .subcommand(
            Cmd::new("completions")
                .about("Generate shell completion script")
                .arg(
                    Arg::new("shell")
                        .long("shell")
                        .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                        .required(true),
                ),
        );
    let matches = cmd.clone().get_matches();

    match matches.subcommand() {
        Some(("dag", sub)) => {
            if sub.subcommand_name().is_none() {
                let mut dag = dag_command();
                let _ = dag.print_help();
                println!();
                return ExitCode::SUCCESS;
            }
            match std::panic::catch_unwind(AssertUnwindSafe(|| dag_run(sub))) {
                Ok(Ok(code)) => code,
                Ok(Err(code)) => code,
                Err(_) => {
                    eprintln!("internal error: unexpected panic");
                    ExitCode::from(1)
                }
            }
        }
        Some(("completions", sub)) => {
            let shell = sub.get_one::<String>("shell").map_or("", String::as_str);
            match shell {
                "bash" => generate(shells::Bash, &mut cmd, "bijux", &mut std::io::stdout()),
                "zsh" => generate(shells::Zsh, &mut cmd, "bijux", &mut std::io::stdout()),
                "fish" => generate(shells::Fish, &mut cmd, "bijux", &mut std::io::stdout()),
                "elvish" => generate(shells::Elvish, &mut cmd, "bijux", &mut std::io::stdout()),
                "powershell" => generate(
                    shells::PowerShell,
                    &mut cmd,
                    "bijux",
                    &mut std::io::stdout(),
                ),
                _ => return ExitCode::from(2),
            }
            ExitCode::SUCCESS
        }
        _ => ExitCode::from(2),
    }
}
