use std::process::ExitCode;

fn main() -> ExitCode {
    let argv = std::env::args().skip(1).collect::<Vec<_>>();
    if argv.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        println!("{}", {{cookiecutter.crate_name}}::help_text());
        return ExitCode::SUCCESS;
    }

    match serde_json::to_string_pretty(&{{cookiecutter.crate_name}}::run(&argv)) {
        Ok(rendered) => {
            println!("{rendered}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to render plugin payload: {error}");
            ExitCode::from(1)
        }
    }
}
