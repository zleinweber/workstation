//! Content-only validation: no filesystem access or action execution.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub version: u64,
    #[serde(default, deserialize_with = "description")]
    pub description: Option<String>,
    pub actions: Vec<Action>,
}

// Missing and null descriptions are different: only omission is allowed.
fn description<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Action {
    Link {
        id: String,
        source: String,
        target: String,
        #[serde(default)]
        conflict: Conflict,
    },
}

impl Action {
    pub fn id(&self) -> &str {
        match self {
            Self::Link { id, .. } => id,
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Conflict {
    #[default]
    Error,
    Backup,
}

/// Parse one YAML recipe, rejecting unknown and duplicate keys before validation.
pub fn parse_recipe(yaml: &str) -> Result<Recipe, String> {
    // Value rejects duplicate keys even inside an internally tagged action.
    // Deserializing from Value also keeps string fields strictly typed.
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(yaml).map_err(|e| e.to_string())?;
    if let Some(actions) = value.get("actions") {
        if !actions.is_sequence() {
            return Err("actions must be a list (use [] for an empty list)".into());
        }
    }
    let recipe: Recipe = serde_yaml_ng::from_value(value).map_err(|e| e.to_string())?;
    if recipe.version != 1 {
        return Err(format!(
            "unsupported recipe version {}; expected 1",
            recipe.version
        ));
    }
    if let Some(description) = &recipe.description {
        nonblank("description", description)?;
    }
    let mut ids = HashMap::new();
    for (index, action) in recipe.actions.iter().enumerate() {
        let position = index + 1;
        nonblank(&format!("action {position} id"), action.id())?;
        if let Some(previous) = ids.insert(action.id(), position) {
            return Err(format!(
                "duplicate action id {:?} at actions {previous} and {position}",
                action.id()
            ));
        }
        match action {
            Action::Link { source, target, .. } => {
                nonblank(&format!("action {position} source"), source)?;
                nonblank(&format!("action {position} target"), target)?;
            }
        }
    }
    Ok(recipe)
}

fn nonblank(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must contain non-whitespace text"))
    } else {
        Ok(())
    }
}
