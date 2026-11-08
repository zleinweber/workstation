//! Argument parsing, independent of repository access.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wstn", version, about = "A recipe-driven dotfiles manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List recipe names and descriptions
    List(RepositoryInputs),
    /// Load and validate a recipe (description rendering is not implemented yet)
    Describe(RecipeInputs),
}

#[derive(Debug, Args)]
pub struct RecipeInputs {
    /// Recipe name from recipes/<name>.yaml
    #[arg(env = "WSTN_RECIPE", hide_env_values = true)]
    pub recipe: Option<String>,

    /// Dotfiles repository (defaults to ~/.dotfiles)
    #[arg(long, env = "WSTN_REPO", hide_env_values = true)]
    pub repo: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RepositoryInputs {
    /// Dotfiles repository (defaults to ~/.dotfiles)
    #[arg(long, env = "WSTN_REPO", hide_env_values = true)]
    pub repo: Option<PathBuf>,
}
