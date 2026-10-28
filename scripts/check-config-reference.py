#!/usr/bin/env python3
"""Check the distributed reference against the Rust model and binding catalog.

The configuration integration tests check typed defaults and parser admission.
This check detects missing leaves that runtime defaulting would otherwise hide.
Neither check establishes rendered UI effects or physical terminal capability.
"""

from pathlib import Path
import os
import re
import shutil
import subprocess
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parent.parent
TABLES = {
    "": "Document",
    "editor": "PartialEditor",
    "scroll": "PartialScroll",
    "windows": "PartialWindows",
    "gutter": "PartialGutter",
    "mouse": "PartialMouse",
    "minibuffer": "PartialMinibuffer",
    "history": "PartialHistory",
    "kill_ring": "PartialKillRing",
    "tree": "PartialTree",
    "syntax": "PartialSyntax",
    "org": "PartialOrg",
    "org.tags": "PartialTags",
    "org.todo": "PartialTodo",
    "org.priority": "PartialPriority",
    "org.time": "PartialTime",
    "org.clock": "PartialClock",
    "org.timer":"PartialTimer",
    "org.duration": "PartialDuration",
    "org.duration.units": "PartialDurationUnits",
    "org.properties": "PartialProperties",
    "files": "PartialFiles",
    "keys": "Keymaps",
    "palette": "PartialPalette",
    "palette.syntax": "PartialSyntaxPalette",
    "palette.gutter": "PartialGutterPalette",
    "palette.org": "PartialOrgPalette",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fields(source, name):
    body = re.search(r"^pub struct " + name + r" \{(.*?)^\}", source, re.M | re.S)
    require(body is not None, f"missing Rust model {name}")
    declarations = re.findall(r"^    pub (?:r#)?(\w+): ([^\n]+),$", body[1], re.M)
    count = len(re.findall(r"^\s+pub\s+(?:r#)?\w+\s*:", body[1], re.M))
    require(len(declarations) == count > 0, f"unrecognized fields in {name}")
    return dict(declarations)


def model(root):
    source = (root / "src/config.rs").read_text()
    source += (root / "src/config/palette.rs").read_text()
    names = set(TABLES.values()) | {"Language"}
    return {name: fields(source, name) for name in names}


def value_at(document, path):
    value = document
    for part in path.split(".") if path else []:
        value = value[part]
    return value


def same_keys(actual, expected, owner):
    missing = sorted(set(expected) - set(actual))
    extra = sorted(set(actual) - set(expected))
    require(not missing and not extra, f"{owner}: missing {missing}, extra {extra}")


def check_schema(document, models):
    for table, name in TABLES.items():
        same_keys(value_at(document, table), models[name], table or "root")
    require(document["language"] == {}, "language overrides must stay optional")
    require(document["files"]["associations"] == {}, "associations must stay optional")
    palette = document["palette"]
    require(len(palette) - 3 + len(palette["syntax"]) + len(palette["gutter"]) + len(palette["org"]) == 46, "expected 46 active RGB roles")


def example(sample, heading):
    lines = iter(sample.splitlines())
    require(any(line.startswith(heading) for line in lines), f"missing {heading}")
    text = []
    for line in lines:
        if not line.startswith("# "):
            break
        text.append(line[2:])
    require(text, f"empty {heading}")
    return tomllib.loads("\n".join(text))


def check_examples(sample, models):
    includes = example(sample, "# Optional includes:")
    same_keys(includes, {"include"}, "include example")
    require(bool(includes["include"]), "include example must name explicit sources")
    languages = example(sample, "# Optional language:")["language"]
    for language in languages.values():
        same_keys(language, models["Language"], "language example")
    associations = example(sample, "# Optional associations:")
    require(bool(associations), "association example must contain rules")


def compiled_bindings(root, context):
    source = (root / "src/input/keymap/defaults.rs").read_text()
    body = re.search(r"const " + context.upper() + r":[^=]+= &\[(.*?)\];", source, re.S)
    require(body is not None, f"missing {context} binding catalog")
    pairs = re.findall(r'\("([^"\n]+)", "([^"\n]+)"\)', body[1])
    require(len(pairs) == len(re.findall(r"^\s*\(", body[1], re.M)), "unrecognized binding entry")
    require(len(dict(pairs)) == len(pairs), "duplicate compiled binding spelling")
    return dict(pairs)


def check_maps(root, document):
    for context, actual in document["keys"].items():
        expected = compiled_bindings(root, context)
        same_keys(actual, expected, f"keys.{context}")
        require(actual == expected, f"keys.{context}: command target drift")


def catalog(text):
    rows = {}
    for line in text.splitlines():
        if not line.startswith("| `"):
            continue
        cells = [cell.strip() for cell in line.split("|")]
        require(len(cells) == 6, f"invalid configuration row: {line}")
        key = cells[1].strip("`")
        require(key not in rows, f"duplicate configuration row: {key}")
        rows[key] = cells[2:5]
    return rows


def leaves(models):
    result = {}
    for table, name in TABLES.items():
        prefix = table + "." if table else ""
        result.update({prefix + field: kind for field, kind in models[name].items() if kind not in models})
    result.update({f"language.<id>.{key}": kind for key, kind in models["Language"].items()})
    return result


def type_label(kind):
    labels = {
        "Option<i64>": "Integer",
        "Option<f64>": "Finite float",
        "Option<bool>": "Boolean",
        "Option<String>": "RGB string",
        "Option<Vec<String>>": "Array",
        "Option<IndentStyle>": "String:",
        "Option<Match>": "String:",
        "Option<TreeRoot>": "String:",
        "Option<Newline>": "String:",
        "Option<LocalTimePolicy>": "String:",
        "Option<RepeatLogging>": "String:",
        "Option<Grammar>": "String:",
        "Option<crate::gutter::Mode>": "String:",
        "Option<crate::gutter::CurrentLine>": "String:",
        "Option<crate::gutter::Side>": "String:",
    }
    if kind.startswith("BTreeMap<"):
        return "Map"
    require(kind in labels, f"unrecognized configuration type: {kind}")
    return labels[kind]


def check_default(document, key, row):
    default = row[1]
    if key.startswith("language.<id>."):
        require(not default.startswith("`"), f"{key}: document inherited language defaults")
        return
    if key.startswith("keys."):
        count = len(value_at(document, key))
        require(default == "Compiled map" and f"all {count} entries" in row[2], f"{key}: map documentation drift")
        return
    require(default.startswith("`") and default.endswith("`"), f"{key}: missing literal default")
    declared = tomllib.loads("value = " + default[1:-1])["value"]
    actual = value_at(document, key)
    require(type(declared) is type(actual) and declared == actual, f"{key}: default drift")


def check_catalog(document, models, rows):
    expected = leaves(models)
    same_keys(rows, expected, "configuration catalog")
    for key, row in rows.items():
        label = "String" if key in ("org.todo.log_drawer", "org.clock.drawer", "org.timer.default") or (key.startswith("org.priority.") or key == "org.time.zone") else type_label(expected[key])
        require(row[0].startswith(label), f"{key}: type documentation drift")
        check_default(document, key, row)


def enum_choices(source, name):
    body = re.search(r"^pub enum " + name + r" \{(.*?)^\}", source, re.M | re.S)
    require(body is not None, f"missing enum {name}")
    variants = re.findall(r"^    (\w+),$", body[1], re.M)
    lines = [line for line in body[1].splitlines() if line.strip() and not line.strip().startswith("#[")]
    require(len(variants) == len(lines) > 0, f"unrecognized enum {name}")
    return [re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower() for name in variants]


def check_enums(root, models, rows):
    source = (root / "src/config.rs").read_text()
    source += (root / "src/gutter/policy.rs").read_text()
    for key, kind in leaves(models).items():
        if type_label(kind) != "String:":
            continue
        expected = enum_choices(source, kind.removeprefix("Option<").removesuffix(">").removeprefix("crate::gutter::"))
        actual = re.findall(r'"([^"\n]+)"', rows[key][0])
        require(sorted(actual) == sorted(expected), f"{key}: enum documentation drift")


def check_links(root):
    paths = ["README.md", "CHANGELOG.md", "docs/reference/index.md", "docs/reference/defaults.md", "docs/reference/bindings.md", "docs/reference/syntax.md", "docs/reference/rectangles.md"]
    for name in paths:
        path = root / name
        for target in re.findall(r"\]\(([^)]+)\)", path.read_text()):
            check_link(path, target)


def check_link(path, target):
    if "://" in target:
        return
    name, _, anchor = target.partition("#")
    destination = path.parent / name if name else path
    require(destination.is_file(), f"{path.name}: missing link {target}")
    if anchor:
        headings = re.findall(r"^#+ (.+)$", destination.read_text(), re.M)
        slugs = [re.sub(r"[^\w -]", "", title.lower()).replace(" ", "-") for title in headings]
        require(anchor in slugs, f"{path.name}: missing anchor {target}")


def packaged(root):
    subprocess.run(["cargo", "build", "--quiet", "--example", "configuration-template"],
                   cwd=root, check=True, timeout=180)
    binary = root / "target/debug/examples/configuration-template"
    with tempfile.TemporaryDirectory(prefix="rev-template-") as directory:
        fixture = Path(directory)
        standalone = fixture / "template"
        shutil.copy2(binary, standalone)
        environment = dict(os.environ, HOME=str(fixture / "home"),
                           XDG_CONFIG_HOME=str(fixture / "user"),
                           XDG_DATA_HOME=str(fixture / "data"),
                           XDG_STATE_HOME=str(fixture / "state"),
                           XDG_CACHE_HOME=str(fixture / "cache"))
        result = subprocess.run([str(standalone)], cwd=fixture, env=environment,
                                stdin=subprocess.DEVNULL, capture_output=True,
                                check=True, timeout=5)
        require(not result.stderr, "packaged template emitted diagnostics")
        require(0 < len(result.stdout) <= 256 * 1024, "packaged template source bound")
        require({entry.name for entry in fixture.iterdir()} == {"template"},
                "packaged source created files or directories")
        return result.stdout


def main():
    reference = (ROOT / "docs/reference/config.toml").read_bytes()
    asset = packaged(ROOT)
    require(asset == reference, "packaged/reference template bytes differ")
    sample = asset.decode("utf-8")
    document = tomllib.loads(sample)
    models = model(ROOT)
    rows = catalog((ROOT / "docs/reference/defaults.md").read_text())
    check_schema(document, models)
    check_examples(sample, models)
    check_maps(ROOT, document)
    check_catalog(document, models, rows)
    check_enums(ROOT, models, rows)
    check_links(ROOT)
    bindings = sum(len(entries) for entries in document["keys"].values())
    print(f"Configuration reference: {len(rows)} schema/catalog entries, 46 active RGB roles, {bindings} default bindings and three optional examples agree; links valid.")
    print(f"Packaged template: {len(asset)} exact reference bytes; copied native binary works without runtime sample files or HOME/XDG creation.")
    print("Typed defaults and published parser bounds are checked by cargo test --test configuration; UI effects require separate native/terminal evidence.")


if __name__ == "__main__":
    main()
