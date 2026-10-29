use super::fixture::Fixture;
use rev::config::{ExportBackend, prepare};
// Configuration selects native data-only formats and finite expansion budgets;
// arrays replace by leaf and the configured default must remain enabled.
#[test]
fn export_configuration() {
    let f = Fixture::new();
    let base=f.write("base.toml","org.export.backends=['org','markdown']\norg.export.default_format='org'\norg.limits.include_depth=3");
    let p=f.write("config.toml","include=['base.toml']\norg.export.default_format='markdown'\norg.limits.include_files=2\norg.limits.expansion_bytes=100\norg.limits.export_bytes=200");
    let startup = prepare(&p);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let org = &startup.snapshot.settings().org;
    assert_eq!(org.export.default_format, ExportBackend::Markdown);
    assert_eq!(
        org.export.backends,
        vec![ExportBackend::Org, ExportBackend::Markdown]
    );
    assert_eq!(org.limits.include_depth, 3);
    assert_eq!(org.limits.include_files, 2);
    assert_eq!(org.limits.expansion_bytes, 100);
    assert_eq!(org.limits.export_bytes, 200);
    assert_eq!(
        startup.snapshot.origin(&["org", "limits", "include_depth"]),
        Some(base.as_path())
    );
    for (name, source) in [
        ("plugin", "org.export.default_format='plugin'"),
        ("tool", "org.export.backends=['pdf']"),
        ("missing default", "org.export.backends=['org']"),
        ("duplicate", "org.export.backends=['html','html']"),
        ("empty", "org.export.backends=[]"),
        ("zero depth", "org.limits.include_depth=0"),
        ("over depth", "org.limits.include_depth=65"),
        ("over files", "org.limits.include_files=513"),
        ("over expansion", "org.limits.expansion_bytes=8388609"),
        ("over output", "org.limits.export_bytes=134217729"),
    ] {
        let p = f.write("invalid.toml", source);
        assert!(prepare(&p).diagnostic.is_some(), "{name}");
    }
}
