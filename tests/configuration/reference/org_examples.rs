use rev::config::{Settings, prepare};

use super::{Fixture, example};

const EXAMPLES: [(&str, &str, &str); 6] = [
    ("agenda", "agenda", "views"),
    ("capture", "capture", "templates"),
    ("publishing", "publish", "projects"),
    ("source", "source", "programs"),
    ("feeds", "feeds", "sources"),
    ("tools", "tools", "fetch"),
];

fn source(name: &str) -> String {
    example(&format!("# Optional Org {name}:"))
}

fn records(settings: &Settings) -> [usize; 6] {
    let org = &settings.org;
    [
        org.agenda.views.len(),
        org.capture.templates.len(),
        org.publish.projects.len(),
        org.source.programs.len(),
        org.feeds.sources.len(),
        org.tools.fetch.len(),
    ]
}

// Uncommented examples must survive the real layered loader with their own
// provenance, while absent data/tools cannot cause startup IO or fallback.
#[test]
fn declared_examples() {
    for (index, (name, table, leaf)) in EXAMPLES.into_iter().enumerate() {
        let fixture = Fixture::new();
        let included = fixture.write("example.toml", &source(name));
        let root = fixture.write("config.toml", "include = ['example.toml']");
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        assert_eq!(
            records(startup.snapshot.settings())[index],
            [1, 1, 1, 1, 2, 7][index],
            "{name}"
        );
        assert_eq!(
            startup.snapshot.origin(&["org", table, leaf]),
            Some(included.as_path())
        );
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 2, "{name}");
        let reset = format!("include = ['example.toml']\norg.{table}.{leaf} = []");
        std::fs::write(&root, reset).unwrap();
        let reset = prepare(&root);
        assert!(reset.diagnostic.is_none(), "{name}: {:?}", reset.diagnostic);
        assert_eq!(records(reset.snapshot.settings())[index], 0, "{name}");
        assert_eq!(
            reset.snapshot.origin(&["org", table, leaf]),
            Some(root.as_path())
        );
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 2, "{name}");
    }
}

// Loading all examples together must publish the advertised typed values,
// including literal argv boundaries and native record discriminants.
#[test]
fn combined_examples() {
    let fixture = Fixture::new();
    let text = EXAMPLES.map(|(name, _, _)| source(name)).join("\n");
    let root = fixture.write("config.toml", &text);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let org = &startup.snapshot.settings().org;
    assert_eq!(org.agenda.views[0].blocks.len(), 2);
    assert_eq!(org.agenda.views[0].blocks[1].query, "TODO");
    assert_eq!(org.capture.templates[0].target.file(), "notes/inbox.org");
    assert_eq!(org.capture.templates[0].text(), "* TODO %?\n  %u\n  %a");
    assert_eq!(org.publish.projects[0].formats.len(), 2);
    assert!(!org.publish.projects[0].upload);
    assert_eq!(org.source.programs[0].language, "python");
    assert_eq!(org.source.programs[0].seconds, 5);
    assert_eq!(org.source.programs[0].output_bytes, 65536);
    assert_eq!(org.source.library_files, ["blocks/library.org"]);
    assert!(org.source.confirm_execution);
    assert!(!org.feeds.sources[0].remote());
    assert!(org.feeds.sources[1].remote());
    assert_eq!(
        org.feeds.sources[1].transport.as_ref(),
        Some(&org.tools.fetch)
    );
    assert_eq!(org.tools.viewer, ["xdg-open", "{target}"]);
    assert_eq!(org.tools.pdf.last().unwrap(), "{input}");
    assert_eq!(org.tools.upload[2], "{output_dir}");
    assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
    assert_eq!(std::fs::read_to_string(root).unwrap(), text);
}

// A typo inside each declared record must reject the candidate; it cannot be
// ignored as an inert hook or silently fall back to an empty declaration.
#[test]
fn closed_records() {
    for (name, _, _) in EXAMPLES.into_iter().take(5) {
        let text = source(name).replacen('{', "{ startup_hook = 'unsupported',", 1);
        let error =
            rev::config::parse(std::path::Path::new("example.toml"), text.as_bytes()).unwrap_err();
        assert!(
            error.to_string().contains("startup_hook"),
            "{name}: {error}"
        );
    }
}
