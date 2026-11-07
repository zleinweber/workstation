use workstation::config::{Action, Conflict, parse_recipe};

#[test]
fn empty_actions_and_omitted_description_are_valid() {
    let recipe = parse_recipe(include_str!("../examples/recipes/minimal.yaml")).unwrap();
    assert_eq!(recipe.version, 1);
    assert_eq!(recipe.description, None);
    assert!(recipe.actions.is_empty());
}

#[test]
fn links_preserve_order_strings_and_defaults() {
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
    for description in ["null", "''", "'  '", "42", "true", "[]", "{}"] {
        let yaml = format!("version: 1\ndescription: {description}\nactions: []");
        assert!(parse_recipe(&yaml).is_err(), "accepted {yaml:?}");
    }
}

#[test]
fn rejects_invalid_actions() {
    let valid =
        "id: shell\n    type: link\n    source: bundles/zsh\n    target: '~/Home with spaces'";
    let wrap = |action: &str| format!("version: 1\nactions:\n  - {action}\n");
    assert!(parse_recipe(&wrap(valid)).is_ok());
    for field in ["id", "type", "source", "target"] {
        let lines: Vec<_> = valid
            .lines()
            .filter(|line| !line.trim_start().starts_with(&format!("{field}:")))
            .collect();
        assert!(
            parse_recipe(&wrap(lines.join("\n").trim_start())).is_err(),
            "accepted missing {field}"
        );
        for value in ["''", "'  '", "null", "42", "true", "[]", "{}"] {
            let lines: Vec<_> = valid
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with(&format!("{field}:")) {
                        format!(
                            "{}{}: {value}",
                            if line.starts_with(' ') { "    " } else { "" },
                            field
                        )
                    } else {
                        line.to_owned()
                    }
                })
                .collect();
            let yaml = wrap(&lines.join("\n"));
            assert!(parse_recipe(&yaml).is_err(), "accepted {yaml:?}");
        }
    }
    for extra in [
        "extra: true",
        "conflict: overwrite",
        "source: other",
        "type: link",
        "id: again",
        "conflict: error\n    conflict: backup",
    ] {
        assert!(
            parse_recipe(&wrap(&format!("{valid}\n    {extra}"))).is_err(),
            "accepted {extra}"
        );
    }
    assert!(parse_recipe(&wrap(&valid.replace("type: link", "type: script"))).is_err());
}

#[test]
fn duplicate_ids_report_both_positions_and_are_case_sensitive() {
    let yaml = include_str!("../examples/recipes/links.yaml");
    let error = parse_recipe(&yaml.replace("editor-config", "shell-config")).unwrap_err();
    assert!(error.contains("shell-config"), "{error}");
    assert!(error.contains("1 and 2"), "{error}");
    assert!(parse_recipe(&yaml.replace("editor-config", "Shell-config")).is_ok());
}
