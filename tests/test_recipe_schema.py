"""Editor schema checks; run with python -m unittest discover -s tests."""

import copy
import json
from pathlib import Path
import unittest

from jsonschema import Draft202012Validator
import yaml


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "schemas/recipe-v1.schema.json").read_text())
VALIDATOR = Draft202012Validator(SCHEMA)
LINK = {"id": "shell", "type": "link", "source": "bundles/zsh", "target": "~/"}


class RecipeSchemaTests(unittest.TestCase):
    def assert_invalid(self, value):
        self.assertFalse(VALIDATOR.is_valid(value), repr(value))

    def test_schema_and_examples(self):
        Draft202012Validator.check_schema(SCHEMA)
        examples = sorted((ROOT / "examples/recipes").glob("*.yaml"))
        self.assertTrue(examples)
        for path in examples:
            with self.subTest(example=path.name):
                VALIDATOR.validate(yaml.safe_load(path.read_text()))

    def test_valid_values(self):
        VALIDATOR.validate({"version": 1, "actions": []})
        for policy in [None, "error", "backup"]:
            link = dict(LINK, target="~/Home with spaces")
            if policy is not None:
                link["conflict"] = policy
            VALIDATOR.validate({"version": 1, "description": "  useful text  ", "actions": [link]})

    def test_invalid_envelopes(self):
        for value in [None, [], {}, {"version": 1}, {"actions": []}]:
            self.assert_invalid(value)
        for version in [None, "1", True, 0, 2, 1.5]:
            self.assert_invalid({"version": version, "actions": []})
        for actions in [None, {}, "", True, 1, [None], [[]]]:
            self.assert_invalid({"version": 1, "actions": actions})
        self.assert_invalid({"version": 1, "actions": [], "extra": True})

    def test_invalid_strings_and_types(self):
        for value in [None, "", "  ", "\t\n", "\u2003", 42, True, [], {}]:
            self.assert_invalid({"version": 1, "actions": [], "description": value})
            for field in ["id", "source", "target", "type", "conflict"]:
                with self.subTest(field=field, value=value):
                    self.assert_invalid({"version": 1, "actions": [dict(LINK, **{field: value})]})

    def test_invalid_actions(self):
        for field in LINK:
            link = copy.copy(LINK)
            del link[field]
            self.assert_invalid({"version": 1, "actions": [link]})
        for changes in [{"type": "script"}, {"type": "download"}, {"conflict": "overwrite"},
                        {"extra": True}, {"when": {"os": ["linux"]}}]:
            self.assert_invalid({"version": 1, "actions": [dict(LINK, **changes)]})


if __name__ == "__main__":
    unittest.main()
