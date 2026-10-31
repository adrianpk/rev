use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use rev::command::Command;
use tree_sitter::{Node, Parser};

use super::{Result, require};

type Models = BTreeMap<String, Vec<(String, String)>>;
type Rows = BTreeMap<String, [String; 3]>;

pub fn check(root: &Path) -> Result<()> {
    let live = Command::ALL
        .into_iter()
        .map(|command| command.name().to_owned())
        .collect();
    for (core, org, column) in [
        ("commands.md", "org/commands.md", None),
        ("bindings.md", "org/bindings.md", Some(2)),
    ] {
        let sources = read_pair(root, core, org)?;
        same_names(core, names(&sources, column), &live)?;
    }
    let sources = read_pair(root, "defaults.md", "org/configuration.md")?;
    let count = check_configuration(root, &sources)?;
    println!(
        "Catalogs passed: {} commands in each combined inventory, {count} configuration entries against the live schema and packaged defaults.",
        Command::ALL.len()
    );
    Ok(())
}

fn read_pair(root: &Path, core: &str, org: &str) -> Result<[String; 2]> {
    let reference = root.join("docs/reference");
    Ok([
        fs::read_to_string(reference.join(core))?,
        fs::read_to_string(reference.join(org))?,
    ])
}

fn names(sources: &[String], column: Option<usize>) -> Vec<String> {
    sources
        .iter()
        .flat_map(|source| {
            let inventory = source
                .split("## Configuration keymap contracts")
                .next()
                .unwrap_or(source);
            inventory
                .lines()
                .filter(|line| line.starts_with('|'))
                .map(|line| column.map_or(line, |index| line.split('|').nth(index).unwrap_or("")))
                .flat_map(|line| line.split('`').skip(1).step_by(2).map(str::to_owned))
        })
        .collect()
}

fn same_names(owner: &str, names: Vec<String>, expected: &BTreeSet<String>) -> Result<()> {
    let actual: BTreeSet<_> = names.iter().cloned().collect();
    require(
        actual.len() == names.len(),
        format!("{owner}: duplicate catalog entry"),
    )?;
    let missing: Vec<_> = expected.difference(&actual).collect();
    let extra: Vec<_> = actual.difference(expected).collect();
    require(
        actual == *expected,
        format!("{owner}: missing {missing:?}, extra {extra:?}"),
    )
}

fn configuration_rows(sources: &[String]) -> Result<Rows> {
    let mut rows = BTreeMap::new();
    for line in sources
        .iter()
        .flat_map(|source| source.lines())
        .filter(|line| line.starts_with("| `"))
    {
        let cells: Vec<_> = line.split('|').map(str::trim).collect();
        require(
            cells.len() == 6,
            format!("invalid configuration row: {line}"),
        )?;
        let key = cells[1].trim_matches('`').to_owned();
        let row = [
            cells[2].to_owned(),
            cells[3].to_owned(),
            cells[4].to_owned(),
        ];
        require(
            rows.insert(key.clone(), row).is_none(),
            format!("duplicate configuration row: {key}"),
        )?;
    }
    Ok(rows)
}

fn check_configuration(root: &Path, sources: &[String]) -> Result<usize> {
    let rows = configuration_rows(sources)?;
    let expected = schema(&models(root)?)?;
    same_names("configuration", rows.keys().cloned().collect(), &expected)?;
    let sample: toml::Value = toml::from_str(rev::config::default_template())?;
    require(
        fs::read(root.join("docs/reference/config.toml"))?
            == rev::config::default_template().as_bytes(),
        "packaged/reference sample mismatch",
    )?;
    for (key, row) in &rows {
        check_default(&sample, key, row)?;
    }
    Ok(rows.len())
}

fn value_at<'a>(sample: &'a toml::Value, key: &str) -> Result<&'a toml::Value> {
    let mut value = sample;
    for part in key.split('.') {
        value = value
            .get(part)
            .ok_or_else(|| format!("missing sample value: {key}"))?;
    }
    Ok(value)
}

fn check_default(sample: &toml::Value, key: &str, row: &[String; 3]) -> Result<()> {
    if key.starts_with("language.<id>.") {
        return require(
            !row[1].starts_with('`'),
            format!("{key}: inherited language default required"),
        );
    }
    if key == "bookmarks.file" {
        return require(
            row[1] == "Unset"
                && sample
                    .get("bookmarks")
                    .and_then(|value| value.get("file"))
                    .is_none(),
            "bookmarks.file: default must remain unset in the sample",
        );
    }
    let actual = value_at(sample, key)?;
    if key.starts_with("keys.") {
        let count = actual.as_table().ok_or("keymap must be a table")?.len();
        return require(
            row[1] == "Compiled map" && row[2].contains(&format!("all {count} entries")),
            format!("{key}: compiled map count drift"),
        );
    }
    let literal = row[1]
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .ok_or_else(|| format!("{key}: missing literal default"))?;
    let declared: toml::Value = toml::from_str(&format!("value = {literal}"))?;
    require(
        declared.get("value") == Some(actual),
        format!("{key}: default drift"),
    )
}

fn models(root: &Path) -> Result<Models> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    let mut models = BTreeMap::new();
    for name in [
        "src/config.rs",
        "src/config/palette.rs",
        "src/config/bookmarks.rs",
        "src/config/export.rs",
        "src/config/publishing.rs",
        "src/config/source.rs",
        "src/config/feeds.rs",
    ] {
        let source = fs::read_to_string(root.join(name))?;
        let tree = parser
            .parse(&source, None)
            .ok_or("schema source parse canceled")?;
        require(
            !tree.root_node().has_error(),
            format!("invalid schema source: {name}"),
        )?;
        add_models(tree.root_node(), &source, &mut models)?;
    }
    Ok(models)
}

fn add_models(root: Node<'_>, source: &str, models: &mut Models) -> Result<()> {
    for node in root
        .named_children(&mut root.walk())
        .filter(|node| node.kind() == "struct_item")
    {
        let name = node
            .child_by_field_name("name")
            .ok_or("missing model name")?;
        let body = node
            .child_by_field_name("body")
            .ok_or("missing model fields")?;
        let fields = body
            .named_children(&mut body.walk())
            .filter(|field| field.kind() == "field_declaration")
            .map(|field| model_field(field, source))
            .collect::<Result<_>>()?;
        require(
            models
                .insert(source[name.byte_range()].to_owned(), fields)
                .is_none(),
            "duplicate schema model",
        )?;
    }
    Ok(())
}

fn model_field(field: Node<'_>, source: &str) -> Result<(String, String)> {
    let name = field
        .child_by_field_name("name")
        .ok_or("missing schema field name")?;
    let kind = field
        .child_by_field_name("type")
        .ok_or("missing schema field type")?;
    Ok((
        source[name.byte_range()]
            .trim_start_matches("r#")
            .to_owned(),
        source[kind.byte_range()].to_owned(),
    ))
}

fn schema(models: &Models) -> Result<BTreeSet<String>> {
    let mut pending = vec![("Document", String::new())];
    let mut leaves = BTreeSet::new();
    while let Some((name, prefix)) = pending.pop() {
        let fields = models
            .get(name)
            .ok_or_else(|| format!("missing schema model: {name}"))?;
        for (field, kind) in fields {
            let key = format!("{prefix}{field}");
            if models.contains_key(kind) {
                pending.push((kind, format!("{key}.")));
            } else {
                leaves.insert(key);
            }
        }
    }
    let language = models.get("Language").ok_or("missing Language schema")?;
    leaves.extend(
        language
            .iter()
            .map(|(field, _)| format!("language.<id>.{field}")),
    );
    Ok(leaves)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Removing a core or Org entry, duplicating it across the two pages, or
    // naming an unimplemented command must fail the actual combined inventory.
    #[test]
    fn command_drift() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let sources = read_pair(root, "commands.md", "org/commands.md").unwrap();
        let original = names(&sources, None);
        let expected = Command::ALL
            .into_iter()
            .map(|command| command.name().to_owned())
            .collect();
        assert!(same_names("commands", original.clone(), &expected).is_ok());
        for (name, target) in [
            ("missing core", "find-file"),
            ("missing Org", "org-mode"),
            ("duplicate", "org-agenda"),
            ("unknown", "future-command"),
        ] {
            let mut changed = original.clone();
            if name.starts_with("missing") {
                changed.retain(|value| value != target);
            } else {
                changed.push(target.into());
            }
            assert!(same_names(name, changed, &expected).is_err(), "{name}");
        }
    }

    // Mutate real Org rows in memory: defaulting cannot hide a missing leaf,
    // duplicate cross-page entry, unsupported setting or incorrect default.
    #[test]
    fn configuration_drift() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let original = read_pair(root, "defaults.md", "org/configuration.md").unwrap();
        let row = original[1]
            .lines()
            .find(|line| line.starts_with("| `org.enabled` |"))
            .unwrap();
        check_configuration(root, &original).unwrap();
        for name in ["missing", "duplicate", "unknown", "default"] {
            let mut changed = original.clone();
            match name {
                "missing" => changed[1] = changed[1].replace(row, ""),
                "duplicate" => changed[0].push_str(&format!("\n{row}\n")),
                "unknown" => {
                    changed[1].push_str("\n| `org.future` | Boolean | `true` | Unknown. |\n")
                }
                _ => changed[1] = changed[1].replace(row, &row.replace("`true`", "`false`")),
            }
            assert!(check_configuration(root, &changed).is_err(), "{name}");
        }
    }

    // A newly added accepted model field must make the old documentation fail;
    // the expected keys come from the model rather than the sample's defaults.
    #[test]
    fn schema_drift() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let rows =
            configuration_rows(&read_pair(root, "defaults.md", "org/configuration.md").unwrap())
                .unwrap();
        let mut models = models(root).unwrap();
        models
            .get_mut("PartialOrg")
            .unwrap()
            .push(("future".into(), "Option<bool>".into()));
        assert!(
            same_names(
                "configuration",
                rows.keys().cloned().collect(),
                &schema(&models).unwrap()
            )
            .is_err()
        );
    }
}
