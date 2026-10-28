use super::fixture::Fixture;
use rev::config::{LocalTimePolicy, Snapshot, prepare};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};

// Include layering retains actual zone provenance. Explicit UTC drives the
// native operation consumer even with invalid local TZ and absent zone files.
#[test]
fn time_layers() {
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        "org.time.zone='Area/Place'\norg.time.ambiguous='prompt'",
    );
    let root = fixture.write("config.toml", "include=['base.toml']\norg.time.zone='UTC'");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let time = &startup.snapshot.settings().org.time;
    assert_eq!(time.zone, "UTC");
    assert_eq!(time.ambiguous, LocalTimePolicy::Prompt);
    assert_eq!(time.nonexistent, LocalTimePolicy::Prompt);
    assert_eq!(
        startup.snapshot.origin(&["org", "time", "zone"]),
        Some(root.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "time", "ambiguous"]),
        Some(base.as_path())
    );
    let env = ZoneEnvironment {
        localtime: fixture.0.join("absent"),
        zoneinfo: fixture.0.join("absent-zones"),
        tz: Some("invalid local TZ".into()),
    };
    let selected = ZoneSelection::parse(&time.zone).unwrap();
    let operation = ZonedOperation::capture(&selected, &env, || Ok(std::time::UNIX_EPOCH)).unwrap();
    assert_eq!(
        operation.now().format_timestamp(false),
        "[1970-01-01 Thu 00:00]"
    );
    assert_eq!(Snapshot::default().settings().org.time.zone, "local");
}

// Configuration permits only explicit prompt policies and safe named paths.
// Invalid candidates reject completely before unrelated settings project.
#[test]
fn time_rejection() {
    let fixture = Fixture::new();
    for source in [
        "org.time.zone=''",
        "org.time.zone='../Zone'",
        "org.time.zone='/tmp/zone'",
        "org.time.zone='Area//Zone'",
        "org.time.zone='EST5EDT,M3.2.0,M11.1.0'",
        "org.time.ambiguous='earliest'",
        "org.time.nonexistent='shift-forward'",
        "org.time.unknown=true",
        "org.time.zone=12",
        "palette.foreground='bad'\norg.time.zone='UTC'",
    ] {
        let path = fixture.write("config.toml", source);
        let startup = prepare(&path);
        assert!(startup.diagnostic.is_some(), "{source}");
        assert_eq!(startup.snapshot, Snapshot::default());
    }
}
