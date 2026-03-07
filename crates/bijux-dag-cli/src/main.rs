use bijux_dag_app::{dag_command, dag_run};
use clap::{Arg, Command as ClapCommand};
use clap_complete::{generate, shells};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut cmd = ClapCommand ::new("bijux")
        .about("Bijux umbrella CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(dag_command())
        .subcommand(
            ClapCommand ::new("completions")
                .about("Generate shell completion script")
                .arg(
                    Arg::new("shell")
                        .long("shell")
                        .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                        .required(true),
                ),
        )
        .subcommand(ClapCommand ::new("rag").about("Not implemented"))
        .subcommand(ClapCommand ::new("rar").about("Not implemented"));
    let matches = cmd.clone().get_matches();

    match matches.subcommand() {
        Some(("dag", sub)) => match std::panic::catch_unwind(|| dag_run(sub)) {
            Ok(Ok(code)) => code,
            Ok(Err(code)) => code,
            Err(_) => {
                eprintln!("internal error: unexpected panic");
                ExitCode::from(1)
            }
        },
        Some(("completions", sub)) => {
            let shell = sub.get_one::<String>("shell").map_or("", String::as_str);
            match shell {
                "bash" => generate(shells::Bash, &mut cmd, "bijux", &mut std::io::stdout()),
                "zsh" => generate(shells::Zsh, &mut cmd, "bijux", &mut std::io::stdout()),
                "fish" => generate(shells::Fish, &mut cmd, "bijux", &mut std::io::stdout()),
                "elvish" => generate(shells::Elvish, &mut cmd, "bijux", &mut std::io::stdout()),
                "powershell" => {
                    generate(shells::PowerShell, &mut cmd, "bijux", &mut std::io::stdout())
                }
                _ => return ExitCode::from(2),
            }
            ExitCode::SUCCESS
        }
        Some(("rag", _)) | Some(("rar", _)) => {
            eprintln!("not implemented");
            ExitCode::from(2)
        }
        _ => ExitCode::from(2),
    }
}
