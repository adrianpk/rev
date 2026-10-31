use super::*;

// Each bundled systems grammar drives both real native highlighting and manual
// declaration extraction. Acceptance stays one canonical, reversible edit.
#[test]
fn systems_grammars() {
    for (file, mode, document, name) in [
        (
            "source.rs",
            "Rust",
            "fn sample() {}\nfn main() { sam| }",
            "sample",
        ),
        (
            "source.c",
            "C",
            "int sample(void) { return 1; }\nint main(void) { sam|; }",
            "sample",
        ),
        (
            "source.go",
            "Go",
            "package main\nfunc sample() {}\nfunc main() { sam| }",
            "sample",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        let original = text(&session);
        assert_eq!(session.buffer().unwrap().mode(), mode, "{file}");
        let buffer = session.buffer().unwrap();
        assert!(
            !buffer
                .syntax()
                .captures(buffer, 0..buffer.len_bytes())
                .is_empty(),
            "{file}"
        );
        complete(&mut session);
        let completion = session
            .completion()
            .unwrap_or_else(|| panic!("{file}: {}", session.echo()));
        assert_eq!(completion.candidates()[0].text(), name, "{file}");
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            text(&session),
            original.replace("{ sam", &format!("{{ {name}"))
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original);
    }
}

// Other open buffers contribute their actual unsaved revision. Neither an
// unopened sibling nor a different language identity becomes an implicit index.
#[test]
fn open_sources() {
    let (fixture, mut session) = source("fn sample_current() {}\n// sam|");
    let current = fixture.0.join("source.rs");
    let other = fixture.write("other.rs", "fn sample_disk() {}\n");
    fixture.write("unopened.rs", "fn sample_unopened() {}\n");
    let foreign = fixture.write("foreign.go", "package main\nfunc sample_foreign() {}\n");
    session.load(&foreign).unwrap();
    session.load(&other).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.handle(Event::Paste("fn sample_unsaved() {}\n".into()));
    session.load(&current).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    complete(&mut session);
    let names: Vec<_> = session
        .completion()
        .unwrap()
        .candidates()
        .iter()
        .map(|c| c.text())
        .collect();
    assert_eq!(names[0], "sample_current");
    assert!(names.contains(&"sample_disk"));
    assert!(names.contains(&"sample_unsaved"));
    assert!(!names.contains(&"sample_unopened"));
    assert!(!names.contains(&"sample_foreign"));
    assert!(
        !std::fs::read_to_string(other)
            .unwrap()
            .contains("sample_unsaved")
    );
    assert!(session.buffer().unwrap().history().is_empty());
}
