//! Resolve ordered actions for describe and future execution commands.

use std::path::{Path, PathBuf};

use crate::config::{Action, Conflict};
use crate::context::{resolve_source, resolve_target};
use crate::recipe::LoadedRecipe;

#[derive(Debug)]
pub enum ResolvedAction<'a> {
    Link {
        id: &'a str,
        source: PathBuf,
        target: PathBuf,
        conflict: &'a Conflict,
    },
}

pub fn resolve_actions<'a>(
    loaded: &'a LoadedRecipe,
    home: Option<&Path>,
) -> Result<Vec<ResolvedAction<'a>>, String> {
    loaded
        .recipe
        .actions
        .iter()
        .map(|action| match action {
            Action::Link {
                id,
                source,
                target,
                conflict,
            } => {
                let context =
                    |error| format!("recipe {}, action {id:?}: {error}", loaded.path.display());
                Ok(ResolvedAction::Link {
                    id,
                    source: resolve_source(&loaded.repository, Path::new(source))
                        .map_err(context)?,
                    target: resolve_target(Path::new(target), home).map_err(context)?,
                    conflict,
                })
            }
        })
        .collect()
}
