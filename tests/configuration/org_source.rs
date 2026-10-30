use super::fixture::Fixture;
use rev::config::{SourceDelivery, prepare};
const PROFILE: &str =
    "[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\n";
// Program arrays replace by assigning layer, retain executable origins and never
// cause discovery or code execution while configuration is loaded.
#[test]
fn source_profiles() {
    let f = Fixture::new();
    let base = f.write(
        "nested/base.toml",
        &format!("org.source.confirm_execution=false\n{PROFILE}"),
    );
    let root = f.write("config.toml", "include=['nested/base.toml']\n");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let source = &startup.snapshot.settings().org.source;
    assert!(!source.confirm_execution);
    assert_eq!(source.programs.len(), 1);
    assert_eq!(source.programs[0].delivery, SourceDelivery::Stdin);
    assert_eq!(source.programs[0].seconds, 30);
    assert_eq!(
        startup.snapshot.origin(&["org", "source", "programs"]),
        Some(base.as_path())
    );
    f.write(
        "config.toml",
        "include=['nested/base.toml']\norg.source.programs=[]\n",
    );
    assert!(
        prepare(&root)
            .snapshot
            .settings()
            .org
            .source
            .programs
            .is_empty()
    );
}
// Closed typed records bound programs, argv, steps and limits; data cannot
// introduce a new action, dynamic placeholder or executable evaluation hook.
#[test]
fn source_profile_rejections() {
    let f = Fixture::new();
    for (name, source) in [
        ("unknown", format!("{PROFILE}hook='run'")),
        ("action", format!("{PROFILE}action='load'")),
        ("delivery", format!("{PROFILE}delivery='eval'")),
        ("cwd", format!("{PROFILE}cwd='arbitrary'")),
        ("seconds", format!("{PROFILE}seconds=0")),
        ("maximum seconds", format!("{PROFILE}seconds=601")),
        ("output", format!("{PROFILE}output_bytes=16777217")),
        ("duplicate", format!("{PROFILE}{PROFILE}")),
        ("token", format!("{PROFILE}argv=['prefix{{input}}']")),
        ("executable", PROFILE.replace("/bin/sh", "{input}")),
        (
            "steps",
            format!(
                "{PROFILE}steps=[{}]",
                ["{executable='/bin/true'}"; 4].join(",")
            ),
        ),
        ("argv", format!("{PROFILE}argv=[{}]", ["'x'"; 64].join(","))),
    ] {
        let startup = prepare(&f.write("config.toml", &source));
        assert!(startup.diagnostic.is_some(), "{name}");
        assert!(
            startup.snapshot.settings().org.source.programs.is_empty(),
            "{name}"
        );
    }
}
// Library arrays retain their assigning-file origin, replace by layer, reject
// unbounded paths and perform no eager reads or process startup.
#[test]
fn source_libraries() {
    let f = Fixture::new();
    let base = f.write(
        "nested/base.toml",
        "org.source.library_files=['missing.org']",
    );
    let root = f.write("config.toml", "include=['nested/base.toml']");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    assert_eq!(
        startup.snapshot.settings().org.source.library_files,
        ["missing.org"]
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "source", "library_files"]),
        Some(base.as_path())
    );
    f.write(
        "config.toml",
        "include=['nested/base.toml']\norg.source.library_files=[]",
    );
    assert!(
        prepare(&root)
            .snapshot
            .settings()
            .org
            .source
            .library_files
            .is_empty()
    );
    for (name, value) in [
        ("empty", "['']".into()),
        ("count", format!("[{}]", ["'file.org'"; 33].join(","))),
        ("length", format!("['{}']", "x".repeat(4097))),
        ("control", "[\"line\\nfile\"]".into()),
    ] {
        f.write("config.toml", &format!("org.source.library_files={value}"));
        assert!(prepare(&root).diagnostic.is_some(), "{name}");
    }
    assert!(!f.0.join("nested/missing.org").exists());
}
