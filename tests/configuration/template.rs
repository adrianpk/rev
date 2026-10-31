use std::path::Path;

use rev::config::{default_template, parse};

// The shipped bytes must remain complete UTF-8 TOML within the existing root
// source limit, without runtime file lookup.
#[test]
fn packaged_source() {
    let text = default_template();
    assert!(!text.is_empty());
    assert!(text.len() <= 256 * 1024);
    assert_eq!(
        text.as_bytes(),
        include_bytes!("../../docs/reference/config.toml")
    );
    assert!(parse(Path::new("packaged/config.toml"), text.as_bytes()).is_ok());
}

// Defaulting must not hide omitted colors/controls or enable optional examples.
// The normal typed equality and context-map tests resolve these same asset bytes.
#[test]
fn explicit_defaults() {
    let text = default_template();
    let raw: toml::Table = toml::from_str(text).unwrap();
    let palette = raw["palette"].as_table().unwrap();
    let mut roles = 0;
    for table in [
        palette,
        palette["syntax"].as_table().unwrap(),
        palette["gutter"].as_table().unwrap(),
        palette["org"].as_table().unwrap(),
    ] {
        for color in table.values().filter_map(toml::Value::as_str) {
            assert!(rev::config::Rgb::parse(color).is_some());
            roles += 1;
        }
    }
    assert_eq!(roles, 55);
    assert_eq!(palette["gutter"]["bookmark"].as_str(), Some("#e0af68"));
    assert_eq!(raw["gutter"].as_table().unwrap().len(), 7);
    assert_eq!(raw["mouse"].as_table().unwrap().len(), 3);
    assert_eq!(raw["mouse"]["tree_activate_on_click"].as_bool(), Some(true));
    let document = parse(Path::new("packaged/config.toml"), text.as_bytes()).unwrap();
    assert_eq!(document.include, Some(Vec::new()));
    assert!(document.language.is_empty());
    assert!(document.files.associations.is_empty());
}

fn editor(snapshot: rev::config::Snapshot, path: &Path) -> rev::session::Session {
    let mut buffers = rev::buffer::Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(path).unwrap();
    let view = rev::view::View::with_settings(buffers.get(id).unwrap(), &snapshot);
    rev::session::Session::new(
        buffers,
        view,
        rev::view::Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap()
}

fn key(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> crossterm::event::Event {
    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(code, modifiers))
}

fn mouse(kind: crossterm::event::MouseEventKind, column: u16, row: u16) -> crossterm::event::Event {
    crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
        kind,
        column,
        row,
        modifiers: crossterm::event::KeyModifiers::NONE,
    })
}

fn wire(session: &mut rev::session::Session) -> Vec<u8> {
    let mut bytes = Vec::new();
    session.frame().unwrap().write(&mut bytes).unwrap();
    bytes
}

// Prove observable equivalence, not just shared source provenance: packaged and
// absent-file defaults must dispatch native keys, pointers and commands alike.
#[test]
fn native_equivalence() {
    use crossterm::event::{
        Event, KeyCode as K, KeyModifiers as M, MouseButton as B, MouseEventKind as P,
    };
    let fixture = super::fixture::Fixture::new();
    let path = fixture.write("source.txt", &"a\t界e\u{301}\r\n".repeat(30));
    let config = fixture.write("config.toml", default_template());
    let mut packaged = editor(rev::config::prepare(&config).snapshot, &path);
    let mut built_in = editor(rev::config::Snapshot::default(), &path);
    assert_eq!(wire(&mut packaged), wire(&mut built_in));
    let events = [
        mouse(P::ScrollDown, 7, 0),
        mouse(P::Down(B::Left), 7, 1),
        mouse(P::Drag(B::Left), 10, 2),
        mouse(P::Up(B::Left), 11, 2),
        key(K::Char('g'), M::CONTROL),
        key(K::Char('x'), M::ALT),
        Event::Paste("mouse-mode".into()),
        key(K::Enter, M::NONE),
        mouse(P::ScrollDown, 7, 0),
        key(K::Char('X'), M::NONE),
        key(K::Char('x'), M::CONTROL),
        key(K::Char('u'), M::NONE),
        key(K::Char('x'), M::ALT),
        Event::Paste("mouse-mode".into()),
        key(K::Enter, M::NONE),
        key(K::Char('x'), M::ALT),
        Event::Paste("gutter-set-line-numbers-relative".into()),
        key(K::Enter, M::NONE),
    ];
    for event in events {
        assert_eq!(packaged.handle(event.clone()), built_in.handle(event));
        assert_eq!(wire(&mut packaged), wire(&mut built_in));
        assert_eq!(packaged.mouse_enabled(), built_in.mouse_enabled());
    }
    assert_eq!(
        packaged.view().point.position,
        built_in.view().point.position
    );
    assert_eq!(
        packaged.buffer().unwrap().history().len(),
        built_in.buffer().unwrap().history().len()
    );
    assert!(!packaged.buffer().unwrap().is_modified());
    assert_eq!(std::fs::read_to_string(config).unwrap(), default_template());
}
