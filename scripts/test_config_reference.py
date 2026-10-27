#!/usr/bin/env python3
"""Prove the reference check rejects omissions that defaulting could conceal."""

from pathlib import Path
import runpy
import tomllib
import unittest


ROOT = Path(__file__).resolve().parent.parent
CHECK = runpy.run_path(str(ROOT / "scripts/check-config-reference.py"))


class ReferenceDrift(unittest.TestCase):
    # Seed each independent drift in the actual sample/model/catalog, in memory.
    # The owner-specific failure must be reported rather than masked by defaults.
    def test_drift(self):
        for name, owner, key, change, message in [
            ("omitted active leaf", "sample", "editor", lambda value: value.pop("tab_width"), "editor: missing"),
            ("new schema leaf", "model", "PartialEditor", lambda value: value.update(future_width="Option<i64>"), "editor: missing"),
            ("changed command", "sample", "keys.editor", lambda value: value.update({"C-x C-f": "save-buffer"}), "command target drift"),
            ("omitted palette row", "catalog", "", lambda value: value.pop("palette.cursor"), "catalog: missing"),
            ("changed default", "catalog", "editor.tab_width", lambda value: value.__setitem__(1, "`8`"), "default drift"),
            ("wrong type", "catalog", "windows.split_ratio", lambda value: value.__setitem__(0, "Integer 1..=2"), "type documentation drift"),
            ("missing enum choice", "catalog", "minibuffer.match", lambda value: value.__setitem__(0, 'String: "prefix"'), "enum documentation drift"),
        ]:
            with self.subTest(name=name):
                self.reject(owner, key, change, message)

    def reject(self, owner, key, change, message):
        sample = tomllib.loads((ROOT / "docs/reference/config.toml").read_text())
        models = CHECK["model"](ROOT)
        rows = CHECK["catalog"]((ROOT / "docs/reference/defaults.md").read_text())
        data = {"sample": sample, "model": models, "catalog": rows}
        target = data[owner][key] if key and owner != "sample" else data[owner]
        if key and owner == "sample":
            target = CHECK["value_at"](target, key)
        change(target)
        with self.assertRaisesRegex(ValueError, message):
            CHECK["check_schema"](sample, models)
            CHECK["check_maps"](ROOT, sample)
            CHECK["check_catalog"](sample, models, rows)
            CHECK["check_enums"](ROOT, models, rows)


if __name__ == "__main__":
    unittest.main()
