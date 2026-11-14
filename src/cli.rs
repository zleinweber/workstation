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
    /// Describe a recipe's ordered actions and resolved paths
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
