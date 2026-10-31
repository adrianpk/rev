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

// Escaped spellings deduplicate by decoded key identity. Literal acceptance
// escapes quotes/backslashes and keeps dotted table segments distinct.
#[test]
fn escaped_keys() {
    for (file, document, expected) in [
        (
            "escaped.json",
            r#"{"space\u0020key":1,"space key":2,"spa|":0}"#,
            r#""space key""#,
        ),
        (
            "escaped.json",
            r#"{"quote\"key":1,"qu|":0}"#,
            r#""quote\"key""#,
        ),
        (
            "escaped.json",
            r#"{"slash\/key":1,"sla|":0}"#,
            r#""slash/key""#,
        ),
        (
            "escaped.json",
            r#"{"\uD83D\uDE80 launch":1,"🚀|":0}"#,
            "\"🚀 launch\"",
        ),
        (
            "escaped.toml",
            "'space key' = 1\n\"spa|\" = 0\n",
            "\"space key\"",
        ),
        (
            "escaped.toml",
            "\"space\\u0020key\" = 1\n'space key' = 2\n\"spa|\" = 0\n",
            "\"space key\"",
        ),
        (
            "escaped.toml",
            "'quote\"key' = 1\nqu| = 0\n",
            r#""quote\"key""#,
        ),
        ("escaped.toml", "service.port = 80\nother.po| = 0\n", "port"),
        (
            "escaped.toml",
            "[server]\nport = 80\n[\"ser|\"]\n",
            "server",
        ),
        (
            "escaped.toml",
            "[\"server.name\" . http]\nport = 80\n[\"server.name\".ht|]\n",
            "\"server.name\".http",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        let original = text(&session);
        complete(&mut session);
        assert_eq!(
            names(&session),
            [expected],
            "{document}: {}",
            session.echo()
        );
        key(&mut session, KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{document}");
        assert!(text(&session).contains(expected));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original);
    }
}

// Incomplete keys recover only bounded structural context. Valid independent
// observations survive malformed neighbors; unknown schema names never appear.
#[test]
fn incomplete_keys() {
    for (file, document, expected) in [
        ("partial.json", "{\"sample\":1,\n \"sam|", "\"sample\""),
        (
            "partial.json",
            "{\"sample\":1,\"broken\":,\"sam|\":0}",
            "\"sample\"",
        ),
        ("partial.toml", "sample = 1\nbroken ???\nsam|", "sample"),
        ("partial.toml", "sample = 1\n\"sam|", "sample"),
        (
            "partial.toml",
            "[server.http]\nport = 80\n[server.ht|",
            "server.http",
        ),
    ] {
        let (_fixture, mut session) = named_source(file, document, "");
        complete(&mut session);
        assert_eq!(
            names(&session),
            [expected],
            "{document}: {}",
            session.echo()
        );
    }
}

// Opening delimiters bypass the ordinary prefix minimum only after structural
// admission. Quoted keys work with in_strings=false; values never become keys.
#[test]
fn automatic_formats() {
    for (file, document, ch, expected) in [
        ("auto.json", "{\"sample\":1, |}", '"', "\"sample\""),
        ("auto.json", "{\"sample\":1, \"sa|\":0}", 'm', "\"sample\""),
        ("auto.toml", "sample = 1\n|", '"', "sample"),
        ("auto.toml", "[server]\nport = 80\n|", '[', "server"),
        ("auto.toml", "port = 80\nother|", '.', "port"),
    ] {
        let (_fixture, mut session) = named_source(file, document, "completion.delay_ms = 0");
        key(&mut session, KeyCode::Char(ch), KeyModifiers::NONE);
        settle(&mut session);
        assert!(
            names(&session).contains(&expected),
            "{document}: {:?}: {}",
            names(&session),
            session.echo()
        );
    }
    for (file, document) in [
        ("auto.json", "{\"sample\":1,\"value\":\"sa|\"}"),
        ("auto.toml", "sample = 1\nvalue = \"sa|\"\n"),
        ("auto.toml", "sample = 1\nvalue = [\n  \"sa|\"\n]\n"),
    ] {
        let (_fixture, mut session) = named_source(
            file,
            document,
            "completion.delay_ms = 0\ncompletion.in_strings = true",
        );
        key(&mut session, KeyCode::Char('m'), KeyModifiers::NONE);
        settle(&mut session);
        assert!(
            session.completion().is_none(),
            "{document}: {:?}",
            names(&session)
        );
    }
}

// Shared observations use current unsaved revisions. Undoing a donor edit
// retires its old name; unopened siblings and the other format remain absent.
#[test]
fn unsaved_formats() {
    for (suffix, document, disk, addition, expected) in [
        (
            "json",
            "{\"sam|\":0}",
            "{\"sampleDisk\":1}\n",
            "{\"sampleUnsaved\":2}\n",
            "\"sampleUnsaved\"",
        ),
        (
            "toml",
            "sam|",
            "sampleDisk = 1\n",
            "sampleUnsaved = 2\n",
            "sampleUnsaved",
        ),
    ] {
        let name = format!("source.{suffix}");
        let (fixture, mut session) = named_source(&name, document, "");
        let current = fixture.0.join(&name);
        let other = fixture.write(&format!("other.{suffix}"), disk);
        fixture.write(
            &format!("unopened.{suffix}"),
            &addition.replace("Unsaved", "Unopened"),
        );
        let foreign = if suffix == "json" {
            fixture.write("foreign.toml", "sampleForeign = 1")
        } else {
            fixture.write("foreign.json", "{\"sampleForeign\":1}")
        };
        session.load(&foreign).unwrap();
        session.load(&other).unwrap();
        session.execute(Command::Motion(Motion::BufferEnd));
        session.handle(Event::Paste(addition.into()));
        session.load(&current).unwrap();
        restore_point(&mut session, document);
        complete(&mut session);
        assert!(
            names(&session).contains(&expected),
            "{suffix}: {:?}: {}",
            names(&session),
            session.echo()
        );
        assert!(!names(&session).iter().any(|name| name.contains("Unopened")));
        assert!(!names(&session).iter().any(|name| name.contains("Foreign")));
        assert_eq!(std::fs::read_to_string(&other).unwrap(), disk);
        session.load(&other).unwrap();
        session.execute(Command::History(History::Undo));
        session.load(&current).unwrap();
        restore_point(&mut session, document);
        complete(&mut session);
        assert!(
            !names(&session).contains(&expected),
            "{suffix}: stale unsaved name"
        );
    }
}

fn restore_point(session: &mut Session, document: &str) {
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..document[..document.find('|').unwrap()].chars().count() {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
}

// Encoding control characters can expand a short observed key past the display
// budget. That is a limit outcome, never a complete no-match claim or an edit.
#[test]
fn label_limit() {
    for (file, document) in [
        (
            "limit.json",
            format!(r#"{{"{}":1,"|":0}}"#, "\\n".repeat(100)),
        ),
        (
            "limit.toml",
            format!("'{}' = 1\n\"|\" = 0\n", "\t".repeat(100)),
        ),
    ] {
        let (_fixture, mut session) = named_source(file, &document, "");
        let original = text(&session);
        complete(&mut session);
        assert!(session.completion().is_none());
        assert!(
            session.echo().contains("label limit"),
            "{file}: {}",
            session.echo()
        );
        assert_eq!(text(&session), original);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
