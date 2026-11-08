use std::ffi::OsString;
use std::process::ExitCode;

use workstation::repository::{load_recipes, select_repository};

const HELP: &str = "Workstation — a recipe-driven dotfiles manager

Usage: wstn [OPTIONS] [COMMAND]

Commands:
  list            List recipe names and descriptions

Options:
  -h, --help       Print help
  -V, --version    Print version

Run wstn list --help for repository selection options.
Recipe setup actions are not implemented yet.";

const LIST_HELP: &str = "List recipe names and descriptions

Usage: wstn list [--repo <path>]

Options:
      --repo <path>  Dotfiles repository (default: WSTN_REPO, then ~/.dotfiles)
  -h, --help         Print help

WSTN_RECIPE does not filter the list.";

fn run() -> Result<(), String> {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match args.first().and_then(|arg| arg.to_str()) {
        None if args.is_empty() => println!("{HELP}"),
        Some("-h" | "--help") if args.len() == 1 => println!("{HELP}"),
        Some("-V" | "--version") if args.len() == 1 => {
            println!("wstn {}", env!("CARGO_PKG_VERSION"))
        }
        Some("list") => {
            let repository = match &args[1..] {
                [] => None,
                [help] if help == "-h" || help == "--help" => {
                    println!("{LIST_HELP}");
                    return Ok(());
                }
                [option, path]
                    if option == "--repo"
                        && !path.is_empty()
                        && !path.to_string_lossy().starts_with('-') =>
                {
                    Some(path.clone())
                }
                _ => return Err("expected wstn list [--repo <path>]; run wstn list --help".into()),
            };
            let repository = select_repository(
                repository,
                std::env::var_os("WSTN_REPO"),
                std::env::var_os("HOME"),
            )?;
            let recipes = load_recipes(&repository)?;
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
                    // Keep multiline descriptions within a single table row.
                    let description = recipe
                        .description
                        .unwrap_or_default()
                        .replace(['\n', '\r', '\t'], " ");
                    println!("{name:<width$}  {description}");
                }
            }
        }
        _ => return Err("unsupported command or option; run wstn --help".into()),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
