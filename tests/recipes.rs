use workstation::config::{Action, Conflict, parse_recipe};

#[test]
fn empty_actions_and_omitted_description_are_valid() {
    let recipe = parse_recipe(include_str!("../examples/recipes/minimal.yaml")).unwrap();
    assert_eq!(recipe.version, 1);
    assert_eq!(recipe.description, None);
    assert!(recipe.actions.is_empty());
}

#[test]
fn links_preserve_order_and_conflict_policies() {
    let recipe = parse_recipe(include_str!("../examples/recipes/links.yaml")).unwrap();
    assert_eq!(
        recipe.actions.iter().map(Action::id).collect::<Vec<_>>(),
        ["shell-config", "editor-config"]
    );
    assert!(matches!(
        recipe.actions[0],
        Action::Link {
            conflict: Conflict::Error,
            ..
        }
    ));
    assert!(matches!(
        recipe.actions[1],
        Action::Link {
            conflict: Conflict::Backup,
            ..
        }
    ));
}

#[test]
fn description_preserves_whitespace() {
    let recipe = parse_recipe("version: 1\ndescription: '  useful text  '\nactions: []").unwrap();
    assert_eq!(recipe.description.as_deref(), Some("  useful text  "));
}

#[test]
fn rejects_invalid_envelopes() {
    for yaml in [
        "",
        "[]",
        "version: [",
        "actions: []",
        "version: 1",
        "version: 2\nactions: []",
        "version: '1'\nactions: []",
        "version: 1.0\nactions: []",
        "version: true\nactions: []",
        "version: 1\nactions: null",
        "version: 1\nactions: {}",
        "version: 1\nactions: []\nextra: true",
        "version: 1\nversion: 1\nactions: []",
        "version: 1\nactions: []\n---\nversion: 1\nactions: []",
    ] {
        assert!(parse_recipe(yaml).is_err(), "accepted {yaml:?}");
    }
}

#[test]
fn rejects_invalid_descriptions() {
    for description in ["null", "''", "'  '", "42", "true", "[]", "{}"] {
        let yaml = format!("version: 1\ndescription: {description}\nactions: []");
        assert!(parse_recipe(&yaml).is_err(), "accepted {yaml:?}");
    }
}

#[test]
fn rejects_invalid_actions() {
    for action in [
        // Missing required fields.
        "{type: link, source: bundles/zsh, target: '~/'}",
        "{id: shell, source: bundles/zsh, target: '~/'}",
        "{id: shell, type: link, target: '~/'}",
        "{id: shell, type: link, source: bundles/zsh}",
        // Unsupported types, fields, and policies.
        "{id: shell, type: script, source: bundles/zsh, target: '~/'}",
        "{id: shell, type: link, source: bundles/zsh, target: '~/', extra: true}",
        "{id: shell, type: link, source: bundles/zsh, target: '~/', conflict: overwrite}",
        // Duplicate keys.
        "{id: shell, type: link, source: bundles/zsh, target: '~/', source: other}",
        "{id: shell, type: link, source: bundles/zsh, target: '~/', type: link}",
        "{id: shell, type: link, source: bundles/zsh, target: '~/', id: again}",
        "{id: shell, type: link, source: bundles/zsh, target: '~/', conflict: error, conflict: backup}",
    ] {
        let yaml = format!("version: 1\nactions: [{action}]");
        assert!(parse_recipe(&yaml).is_err(), "accepted {yaml:?}");
    }
}

#[test]
fn rejects_invalid_action_field_values() {
    for value in ["''", "'  '", "null", "42", "true", "[]", "{}"] {
        for action in [
            format!("{{id: {value}, type: link, source: bundles/zsh, target: '~/'}}"),
            format!("{{id: shell, type: {value}, source: bundles/zsh, target: '~/'}}"),
            format!("{{id: shell, type: link, source: {value}, target: '~/'}}"),
            format!("{{id: shell, type: link, source: bundles/zsh, target: {value}}}"),
        ] {
            let yaml = format!("version: 1\nactions: [{action}]");
            assert!(parse_recipe(&yaml).is_err(), "accepted {yaml:?}");
        }
    }
}

#[test]
fn link_target_accepts_spaces() {
    let yaml = "version: 1\nactions: [{id: shell, type: link, source: bundles/zsh, target: '~/Home with spaces'}]";
    assert!(parse_recipe(yaml).is_ok());
}

#[test]
fn duplicate_ids_report_both_positions() {
    let yaml = include_str!("../examples/recipes/links.yaml");
    let error = parse_recipe(&yaml.replace("editor-config", "shell-config")).unwrap_err();
    assert!(error.contains("shell-config"), "{error}");
    assert!(error.contains("1 and 2"), "{error}");
}

#[test]
fn action_ids_are_case_sensitive() {
    let yaml = include_str!("../examples/recipes/links.yaml");
    assert!(parse_recipe(&yaml.replace("editor-config", "Shell-config")).is_ok());
}
