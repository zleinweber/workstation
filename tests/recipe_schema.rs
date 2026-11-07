use std::{fs, path::Path};

#[test]
fn editor_schema_accepts_example_recipes() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/recipe-v1.schema.json")).unwrap();
    jsonschema::meta::validate(&schema).expect("valid JSON Schema");
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");

    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/recipes");
    let mut count = 0;
    for entry in fs::read_dir(examples).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let yaml = fs::read_to_string(&path).unwrap();
        let recipe: serde_json::Value = serde_yaml_ng::from_str(&yaml)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        validator
            .validate(&recipe)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        count += 1;
    }
    assert!(count > 0, "expected at least one example recipe");
}
