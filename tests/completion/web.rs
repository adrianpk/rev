use super::*;

// Distinct bundled web parsers supply both frame colors and real declaration
// proposals. Acceptance and undo retain the common checked literal edit owner.
#[test]
fn web_grammars() {
    for (file, mode, document, name) in [
        (
            "source.html",
            "HTML",
            "<article></article>\n<ar|>",
            "article",
        ),
        ("source.css", "CSS", ".sample {}\n.sam| {}", "sample"),
        (
            "source.js",
            "JavaScript",
            "function sample() {}\nfunction main() { sam|; }",
            "sample",
        ),
        (
            "source.ts",
            "TypeScript",
            "function sample(): void {}\nfunction main() { sam|; }",
            "sample",
        ),
        (
            "source.tsx",
            "TSX",
            "function sample(): void {}\nconst view = <div />;\nfunction main() { sam|; }",
            "sample",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        let original = text(&session);
        assert_eq!(session.buffer().unwrap().mode(), mode, "{file}");
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| !row.syntax().is_empty()),
            "{file}"
        );
        complete(&mut session);
        let completion = session
            .completion()
            .unwrap_or_else(|| panic!("{file}: {}", session.echo()));
        assert_eq!(completion.candidates()[0].text(), name, "{file}");
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{file}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original, "{file}");
    }
}
