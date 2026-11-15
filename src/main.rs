use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use workstation::cli::{Cli, Command};
use workstation::config::Conflict;
use workstation::plan::{ResolvedAction, resolve_actions};
use workstation::recipe::{discover_recipes, load_recipe, resolve_repository};

fn run(cli: Cli) -> Result<(), String> {
    let Some(command) = cli.command else {
        Cli::command().print_help().map_err(|e| e.to_string())?;
        println!();
        return Ok(());
    };
    match command {
        Command::List(inputs) => {
            let cwd = std::env::current_dir()
                .map_err(|e| format!("cannot determine invocation directory: {e}"))?;
            let home = std::env::var_os("HOME").map(PathBuf::from);
            let repository = resolve_repository(inputs.repo.as_deref(), &cwd, home.as_deref())?;
            let names = discover_recipes(&repository)?;
            // Validate every recipe before producing output, avoiding partial tables.
            let recipes = names
                .into_iter()
                .map(|name| {
                    load_recipe(repository.clone(), Some(&name)).map(|loaded| (name, loaded.recipe))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if recipes.is_empty() {
                println!("No recipes available in {}.", repository.display());
            } else {
                let width = recipes
                    .iter()
                    .map(|(name, _)| name.len())
                    .max()
                    .unwrap_or(0)
                    .max(4);
                println!("{:<width$}  Description", "Name");
                for (name, recipe) in recipes {
                    let description = recipe
                        .description
                        .unwrap_or_default()
                        .replace(['\n', '\r', '\t'], " ");
                    println!("{name:<width$}  {description}");
                }
            }
            Ok(())
        }
        Command::Describe(inputs) => {
            let cwd = std::env::current_dir()
                .map_err(|e| format!("cannot determine invocation directory: {e}"))?;
            let home = std::env::var_os("HOME").map(PathBuf::from);
            let repository = resolve_repository(inputs.repo.as_deref(), &cwd, home.as_deref())?;
            let loaded = load_recipe(repository, inputs.recipe.as_deref())?;
            let actions = resolve_actions(&loaded, home.as_deref())?;
            if let Some(description) = &loaded.recipe.description {
                println!("{description}");
            }
            if actions.is_empty() {
                println!("No actions.");
            }
            for action in actions {
                match action {
                    ResolvedAction::Link {
                        id,
                        source,
                        target,
                        conflict,
                    } => {
                        let conflict = match conflict {
                            Conflict::Error => "error",
                            Conflict::Backup => "backup",
                        };
                        println!("{id} (link)");
                        println!("  source: {}", source.display());
                        println!("  target: {}", target.display());
                        println!("  conflict: {conflict}");
                    }
                }
            }
            Ok(())
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
