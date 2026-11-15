use crate::support::Result;
use rev::{
    command::{Binding, Command, PromptAction},
    config::{InteractionProfile, Keymaps},
    input::keymap::{self, Context, Sequence},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

type Maps = BTreeMap<String, BTreeMap<Sequence, String>>;

#[derive(Deserialize)]
struct Catalog {
    source_tree: String,
    commands: Vec<Name>,
    actions: Vec<String>,
    bindings: Vec<Row>,
}
#[derive(Deserialize)]
struct Name {
    name: String,
}
#[derive(Deserialize)]
struct Row {
    context: String,
    key: String,
    command: String,
}
#[derive(Deserialize)]
struct Manifest {
    source_tree: String,
    status: String,
    changes: Vec<Change>,
}
#[derive(Deserialize)]
struct Change {
    id: String,
    contexts: Vec<String>,
    inherited_contexts: Vec<String>,
    key: String,
    normalized: String,
    before: BTreeMap<String, Option<String>>,
    after: Option<String>,
}

// Expectations come from the independently reviewed baseline and accepted edits,
// never from today's factories. Exact map equality also protects every untouched leaf.
pub(super) fn run() -> Result<()> {
    let catalog: Catalog = serde_json::from_str(include_str!(
        "../../ops/default/report/rev-factory-shortcuts/catalog.json"
    ))?;
    let manifest: Manifest = serde_json::from_str(include_str!(
        "../../ops/default/report/rev-factory-shortcuts/mapping-proposal.json"
    ))?;
    inventory(&catalog);
    assert_eq!(manifest.source_tree, catalog.source_tree);
    assert_eq!(manifest.status, "accepted");
    assert_eq!(manifest.changes.len(), 29);
    let mut expected = baseline(&catalog)?;
    let mut edits = 0;
    let mut identities = BTreeSet::new();
    for change in &manifest.changes {
        assert!(identities.insert(&change.id));
        edits += apply(&mut expected, change)?;
    }
    assert_eq!(edits, 49);
    add_navigation_maps(&mut expected)?;
    compare_maps(&expected)?;
    println!(
        "PASS accepted manifest: 707 original commands plus three editing commands and five navigation commands, six actions, 28 complete maps, 29 records / 49 base edits, inherited and retired leaves, both profiles; untouched defaults preserved"
    );
    Ok(())
}

fn compare_maps(expected: &Maps) -> Result<()> {
    for profile in [InteractionProfile::NonModal, InteractionProfile::Modal] {
        let actual = keymap::compile_for(&Keymaps::default(), profile)?;
        for context in Context::ALL {
            let wanted = &expected[context.name()];
            let defaults = entries(context.defaults().iter().map(|(key, name)| (*key, *name)))?;
            assert_eq!(&defaults, wanted, "factory {}", context.name());
            let compiled: BTreeMap<_, _> = actual
                .map(context)
                .bindings()
                .map(|(key, target)| (key.clone(), target.name().to_owned()))
                .collect();
            assert_eq!(&compiled, wanted, "{profile:?} {}", context.name());
        }
    }
    Ok(())
}

fn add_navigation_maps(expected: &mut Maps) -> Result<()> {
    for context in [
        "editor",
        "vim_insert",
        "vim_replace",
        "markdown",
        "org",
        "org_capture",
        "org_source_edit",
        "macro_definition",
        "git_commit",
    ] {
        for (key, name) in [
            ("C-M-n", "forward-pair"),
            ("C-M-p", "backward-pair"),
            ("C-M-u", "up-pair"),
            ("C-M-d", "down-pair"),
        ] {
            let old = expected
                .get_mut(context)
                .expect("frozen source context")
                .insert(Sequence::parse(key)?, name.into());
            assert!(old.is_none(), "new navigation leaf {context} {key}");
        }
    }
    Ok(())
}

fn inventory(catalog: &Catalog) {
    let names: BTreeSet<_> = catalog.commands.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names.len(), 707);
    let added = BTreeSet::from([
        "auto-pair-mode",
        "pair-wrap",
        "pair-unwrap",
        "matching-delimiter",
        "forward-pair",
        "backward-pair",
        "up-pair",
        "down-pair",
    ]);
    let live: BTreeSet<_> = Command::ALL.iter().map(Command::name).collect();
    assert!(names.is_disjoint(&added));
    assert!(added.is_subset(&live));
    assert_eq!(live.len(), 715);
    assert_eq!(names, live.difference(&added).copied().collect());
    let actions: BTreeSet<_> = catalog.actions.iter().map(String::as_str).collect();
    assert_eq!(actions.len(), 6);
    assert_eq!(
        actions,
        PromptAction::ALL.into_iter().map(|a| a.name()).collect()
    );
    let matrix = include_str!("../../ops/default/report/rev-factory-shortcuts/semantic-matrix.md");
    let reviewed: BTreeSet<_> = matrix
        .lines()
        .filter_map(|line| {
            line.strip_prefix("| `")?
                .split_once('`')
                .map(|(name, _)| name)
        })
        .collect();
    assert!(names.is_subset(&reviewed));
    assert!(actions.is_subset(&reviewed));
    assert!(!matrix.lines().any(|line| {
        line.split('|')
            .any(|field| field.trim().eq_ignore_ascii_case("unresolved"))
    }));
    for context in Context::ALL {
        assert!(matrix.contains(context.name()));
    }
}

fn baseline(catalog: &Catalog) -> Result<Maps> {
    assert_eq!(catalog.bindings.len(), 2349);
    let mut expected = Maps::new();
    for row in &catalog.bindings {
        let binding = Binding::named(&row.command).ok_or("unknown frozen command")?;
        let target = binding.name();
        let old = expected
            .entry(row.context.clone())
            .or_default()
            .insert(Sequence::parse(&row.key)?, target.into());
        assert!(old.as_deref().is_none_or(|previous| previous == target));
    }
    assert_eq!(expected.len(), 28);
    Ok(expected)
}

fn apply(expected: &mut Maps, change: &Change) -> Result<usize> {
    let key = Sequence::parse(&change.key)?;
    assert_eq!(key.to_string(), change.normalized);
    for context in &change.contexts {
        let map = expected
            .get_mut(context)
            .ok_or("unknown accepted context")?;
        assert_eq!(
            map.get(&key),
            change.before[context].as_ref(),
            "{} {context}",
            change.id
        );
        replace(map, &key, change.after.as_deref());
    }
    for context in &change.inherited_contexts {
        replace(
            expected
                .get_mut(context)
                .ok_or("unknown inherited context")?,
            &key,
            change.after.as_deref(),
        );
    }
    Ok(change.contexts.len())
}

fn replace(map: &mut BTreeMap<Sequence, String>, key: &Sequence, target: Option<&str>) {
    if let Some(target) = target {
        map.insert(key.clone(), target.into());
    } else {
        assert!(map.remove(key).is_some());
    }
}

fn entries<'a>(
    rows: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<BTreeMap<Sequence, String>> {
    rows.map(|(key, name)| {
        Ok((
            Sequence::parse(key)?,
            Binding::named(name)
                .ok_or("unknown factory command")?
                .name()
                .into(),
        ))
    })
    .collect()
}
