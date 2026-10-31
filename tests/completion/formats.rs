use super::*;

fn names(session: &Session) -> Vec<&str> {
    session
        .completion()
        .map(|completion| {
            completion
                .candidates()
                .iter()
                .map(|candidate| candidate.text())
                .collect()
        })
        .unwrap_or_default()
}

// Both formats use actual parser colors and structural observations, then the
// common checked replacement and one-step undo without touching the source early.
#[test]
fn format_grammars() {
    for (file, mode, document, expected) in [
        (
            "source.json",
            "JSON",
            r#"{"sample":1,"sam|":2}"#,
            r#""sample""#,
        ),
        ("source.toml", "TOML", "sample = 1\nsam| = 2\n", "sample"),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        let original = text(&session);
        assert_eq!(session.buffer().unwrap().mode(), mode);
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| !row.syntax().is_empty())
        );
        complete(&mut session);
        assert_eq!(
            names(&session),
            [expected],
            "{document}: {}",
            session.echo()
        );
        assert_eq!(text(&session), original);
        assert!(session.buffer().unwrap().history().is_empty());
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert!(!text(&session).contains("\"sam\""));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original);
    }
}

// Keys nested at any depth are observed once per insertion identity. Values
// never declare names; ordinary values remain quiet even for manual invocation.
#[test]
fn structural_contexts() {
    for (file, document, expected) in [
        (
            "nested.json",
            r#"{"sample":1,"nested":{"sample":2,"sampleOther":3},"sam|":0}"#,
            vec![r#""sample""#, r#""sampleOther""#],
        ),
        (
            "nested.toml",
            "sample = 1\nobject = { sampleOther = 2 }\nsam| = 0\n",
            vec!["sample", "sampleOther"],
        ),
        (
            "nested.toml",
            "[server.http]\nport = 80\n[server.ht|]\n",
            vec!["server.http"],
        ),
        (
            "nested.toml",
            "[[products]]\nname = 'x'\n[[prod|]]\n",
            vec!["products"],
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        complete(&mut session);
        let actual = names(&session);
        assert_eq!(
            actual.len(),
            expected.len(),
            "{document}: {actual:?}: {}",
            session.echo()
        );
        for name in expected {
            assert!(actual.contains(&name), "{document}: {actual:?}");
        }
    }
    for (file, document) in [
        ("quiet.json", r#"{"sample":1,"value":"sam|"}"#),
        ("quiet.json", r#"{"sample":1,"values":["sam|"]}"#),
        ("quiet.json", r#"{"sample":1,"value": tru|}"#),
        ("quiet.toml", "sample = 1\nvalue = 'sam|'\n"),
        ("quiet.toml", "sample = 1\nvalue = [\"sam|\"]\n"),
        ("quiet.toml", "sample = 1\n# sam|\n"),
        ("quiet.toml", "sample = 1\nvalue = \"\"\"\nsam|"),
        ("quiet.toml", "sample = 1\nvalue = [\n\"sam|"),
        ("quiet.json", "{\"sample\":1,\"value\":[\n \"sam|"),
    ] {
        let (_fixture, mut session) = named_source(
            file,
            document,
            "completion.in_strings = true\ncompletion.in_comments = true",
        );
        complete(&mut session);
        assert!(
            session.completion().is_none(),
            "{document}: {:?}",
            names(&session)
        );
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
