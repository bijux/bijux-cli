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
    let mut cmd = dag_command().subcommand(
        Cmd::new("completions").about("Generate shell completion script").arg(
            Arg::new("shell")
                .long("shell")
                .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                .required(true),
        ),
    );
    let matches = cmd.clone().get_matches();

    match matches.subcommand() {
        Some(("completions", sub)) => {
            let shell = sub.get_one::<String>("shell").map_or("", String::as_str);
            match shell {
                "bash" => generate(shells::Bash, &mut cmd, "bijux-dag", &mut std::io::stdout()),
                "zsh" => generate(shells::Zsh, &mut cmd, "bijux-dag", &mut std::io::stdout()),
                "fish" => generate(shells::Fish, &mut cmd, "bijux-dag", &mut std::io::stdout()),
                "elvish" => generate(shells::Elvish, &mut cmd, "bijux-dag", &mut std::io::stdout()),
                "powershell" => {
                    generate(shells::PowerShell, &mut cmd, "bijux-dag", &mut std::io::stdout())
                }
                _ => return ExitCode::from(2),
            }
            ExitCode::SUCCESS
        }
        _ => match std::panic::catch_unwind(AssertUnwindSafe(|| dag_run(&matches))) {
            Ok(Ok(code)) => code,
            Ok(Err(code)) => code,
            Err(_) => {
                eprintln!("internal error: unexpected panic");
                ExitCode::from(1)
            }
        },
    }
}
