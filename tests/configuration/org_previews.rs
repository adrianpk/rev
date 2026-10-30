use super::fixture::Fixture;
use rev::{buffer::Buffers, config::prepare, view::View};
// Both partial leaves retain assigning origin and explicit false; loading an
// enabled view does not read, fetch or create a missing linked resource.
#[test]
fn preview_configuration() {
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        "org.preview.enabled=true\norg.preview.max_bytes=73",
    );
    let path = f.write("config.toml", "include=['base.toml']");
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none());
    assert_eq!(
        startup.snapshot.origin(&["org", "preview", "enabled"]),
        Some(base.as_path())
    );
    assert_eq!(startup.snapshot.settings().org.preview.max_bytes, 73);
    let settings = startup.snapshot;
    let mut buffers = Buffers::with_settings(settings.clone()).unwrap();
    let id = buffers
        .visit(&f.write("source.org", "[[file:absent.png]]"))
        .unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &settings);
    assert!(view.preview.enabled);
    assert!(
        view.preview
            .images(buffers.get(id).unwrap().stamp())
            .is_empty()
    );
    assert!(!f.0.join("absent.png").exists());
    let path = f.write(
        "config.toml",
        "include=['base.toml']\norg.preview.enabled=false",
    );
    let replaced = prepare(&path);
    assert!(replaced.diagnostic.is_none());
    assert!(!replaced.snapshot.settings().org.preview.enabled);
    for (name, source) in [
        ("zero", "org.preview.max_bytes=0"),
        ("large", "org.preview.max_bytes=16777217"),
        ("unknown", "org.preview.fetch=true"),
    ] {
        let path = f.write("invalid.toml", source);
        assert!(prepare(&path).diagnostic.is_some(), "{name}");
    }
}
