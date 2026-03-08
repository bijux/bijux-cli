pub mod cargo;
pub mod git;

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), String>;
}

pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), String> {
        let status = std::process::Command::new(program)
            .args(args)
            .status()
            .map_err(|err| err.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("command failed: {} {}", program, args.join(" ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_command_runner_executes_successful_command() {
        let runner = ProcessCommandRunner;
        runner
            .run("cargo", &["--version"])
            .expect("cargo --version should succeed");
    }
}
