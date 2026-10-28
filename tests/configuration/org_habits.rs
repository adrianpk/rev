use super::fixture::Fixture;
use rev::config::{Snapshot, prepare};
// Every graph preference and semantic background is typed, layered and covered
// by complete-candidate rejection. Factory values match the admitted Org profile.
#[test]
fn habit_configuration() {
    let f = Fixture::new();
    let base=f.write("base.toml","org.habit.preceding_days=5\norg.habit.today_only=false\npalette.org.habit_overdue='#010203'");
    let root = f.write(
        "config.toml",
        "include=['base.toml']\norg.habit.following_days=0\norg.habit.done_always_green=true",
    );
    let s = prepare(&root);
    assert!(s.diagnostic.is_none(), "{:?}", s.diagnostic);
    let h = &s.snapshot.settings().org.habit;
    assert_eq!(
        (h.preceding_days, h.following_days, h.graph_column),
        (5, 0, 40)
    );
    assert!(!h.today_only);
    assert!(!h.show_all_today);
    assert!(h.done_always_green);
    assert_eq!(
        s.snapshot.origin(&["org", "habit", "preceding_days"]),
        Some(base.as_path())
    );
    assert_eq!(
        s.snapshot.origin(&["org", "habit", "following_days"]),
        Some(root.as_path())
    );
    assert_eq!(
        s.snapshot.settings().palette.org.habit_overdue,
        rev::config::Rgb(1, 2, 3)
    );
    let h = Snapshot::default().settings().org.habit.clone();
    assert_eq!(
        (h.preceding_days, h.following_days, h.graph_column),
        (21, 7, 40)
    );
    assert!(h.today_only);
    assert!(!h.done_always_green);
    for (name, source) in [
        ("negative", "org.habit.preceding_days=-1"),
        ("days", "org.habit.following_days=367"),
        ("column", "org.habit.graph_column=4097"),
        ("type", "org.habit.today_only=1"),
        ("color", "palette.org.habit_ready='green'"),
    ] {
        let path = f.write("candidate.toml", source);
        let s = prepare(&path);
        assert!(s.diagnostic.is_some(), "{name}");
        assert_eq!(s.snapshot, Snapshot::default(), "{name}");
    }
}
