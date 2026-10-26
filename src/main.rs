use std::process::ExitCode;

const HELP: &str = "Workstation — a recipe-driven dotfiles manager

Usage: wstn [OPTIONS]

Options:
  -h, --help       Print help
  -V, --version    Print version

This is the initial project scaffold.
Recipe parsing and setup actions are not implemented yet.";

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let first = args.next();

    if args.next().is_some() {
        eprintln!("error: unexpected arguments; run wstn --help");
        return ExitCode::from(2);
    }

    match first.as_deref().and_then(|arg| arg.to_str()) {
        None if first.is_none() => println!("{HELP}"),
        Some("-h" | "--help") => println!("{HELP}"),
        Some("-V" | "--version") => println!("wstn {}", env!("CARGO_PKG_VERSION")),
        _ => {
            eprintln!("error: unsupported command or option; run wstn --help");
            return ExitCode::from(2);
        }
    }

    ExitCode::SUCCESS
}
