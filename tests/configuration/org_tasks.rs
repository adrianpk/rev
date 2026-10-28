use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};

// Task policy participates in normal leaf layering and exact source provenance;
// empty finished arrays and false overrides remain explicit admitted values.
#[test]
fn task_policy_layers() {
    let fixture = Fixture::new();
    let included = fixture.write("base.toml", "[org.todo]\nsequence=['OPEN','FINISHED']\nfinished=['FINISHED']\nlog_done=true\nlog_drawer='LOGBOOK'\nblock_children=true");
    let root = fixture.write(
        "config.toml",
        "include=['base.toml']\n[org.todo]\nfinished=[]\nlog_done=false",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let policy = &startup.snapshot.settings().org.todo;
    assert_eq!(policy.sequence, ["OPEN", "FINISHED"]);
    assert!(policy.finished.is_empty());
    assert!(!policy.log_done);
    assert!(policy.block_children);
    assert_eq!(policy.log_drawer, "LOGBOOK");
    assert_eq!(
        startup.snapshot.origin(&["org", "todo", "finished"]),
        Some(root.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["org", "todo", "log_drawer"]),
        Some(included.as_path())
    );
}

// Invalid keywords, ambiguous finished states and executable-looking drawer
// values reject the whole candidate. The exact 64-keyword cap is admitted.
#[test]
fn task_policy_bounds() {
    let fixture = Fixture::new();
    let words = (0..64)
        .map(|i| format!("'STATE-{i}'"))
        .collect::<Vec<_>>()
        .join(",");
    for (body, valid) in [
        (format!("sequence=[{words}]\nfinished=['STATE-63']"), true),
        (format!("sequence=[{words},'EXTRA']\nfinished=[]"), false),
        ("sequence=[]\nfinished=[]".into(), false),
        ("sequence=['TODO','TODO']".into(), false),
        ("finished=['MISSING']".into(), false),
        ("finished=['DONE','DONE']".into(), false),
        ("sequence=['TODO(x)','DONE']".into(), false),
        ("log_done='true'".into(), false),
        ("log_drawer='PROPERTIES'".into(), false),
        ("log_drawer='END'".into(), false),
        ("log_drawer='x\\n:END:'".into(), false),
        ("unknown=true".into(), false),
    ] {
        let path = fixture.write("config.toml", &format!("[org.todo]\n{body}"));
        let startup = prepare(&path);
        assert_eq!(
            startup.diagnostic.is_none(),
            valid,
            "{body}: {:?}",
            startup.diagnostic
        );
        if !valid {
            assert_eq!(startup.snapshot, Snapshot::default());
        }
    }
}
