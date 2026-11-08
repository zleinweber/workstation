use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use workstation::cli::{Cli, Command};
use workstation::recipe::{load_recipe, resolve_repository};

fn run(cli: Cli) -> Result<(), String> {
    let Some(command) = cli.command else {
        Cli::command().print_help().map_err(|e| e.to_string())?;
        println!();
        return Ok(());
    };
    match command {
        Command::Describe(inputs) => {
            let cwd = std::env::current_dir()
                .map_err(|e| format!("cannot determine invocation directory: {e}"))?;
            let home = std::env::var_os("HOME").map(PathBuf::from);
            let repository = resolve_repository(inputs.repo.as_deref(), &cwd, home.as_deref())?;
            load_recipe(repository, inputs.recipe.as_deref())?;
            Err("describe rendering is not implemented yet".into())
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
