use super::*;
fn rejected(source: &str, command: OrgCommand, diagnostic: &str) {
    let (_fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", source);
    let view = session.view().clone();
    action(&mut session, command);
    assert!(
        session
            .echo()
            .to_lowercase()
            .contains(&diagnostic.to_lowercase()),
        "{}",
        session.echo()
    );
    assert_eq!(text(&session), source);
    assert_eq!(session.view(), &view);
    assert!(session.buffer().unwrap().history().is_empty());
}
// Unknown dynamic writers, executable/unsupported options and malformed column
// definitions cannot replace even the first byte of an existing report body.
#[test]
fn column_rejections() {
    for (name, header, diagnostic) in [
        ("unknown", "unknown :format \"%ITEM\"", "not a clocktable"),
        (
            "formatter",
            "columnview :formatter \"arbitrary\"",
            "Unsupported",
        ),
        (
            "external",
            "columnview :id file:other.org",
            "local or global",
        ),
        ("duplicate", "columnview :id global :id local", "Repeated"),
        ("depth", "columnview :maxlevel 0", "1..256"),
        ("quote", "columnview :format \"%ITEM", "Unclosed"),
        (
            "summary",
            "columnview :format \"%EFFORT{:mean}\"",
            "Unsupported",
        ),
        ("special", "columnview :format \"%ITEM{+}\"", "stored"),
        ("width", "columnview :format \"%25ITEM\"", "unsupported"),
        (
            "duplicate fields",
            "columnview :format \"%ITEM %item\"",
            "distinct",
        ),
    ] {
        let source = format!("#+BEGIN: {header}\nkeep {name}\n#+END:\n* Task\n");
        rejected(&source, OrgCommand::UpdateDynamicBlock, diagnostic);
    }
}
// Late invalid numbers and unrepresentable cell text reject the complete
// candidate. A successful earlier row never leaks into canonical report source.
#[test]
fn column_values_reject() {
    for (name, field, value, diagnostic) in [
        ("number", "POINTS{+}", "not-a-number", "finite numbers"),
        ("nonfinite", "POINTS{+}", "NaN", "not finite"),
        ("duration", "EFFORT{:}", "bad-duration", "duration"),
        ("delimiter", "OWNER", "a|b", "native table cell"),
    ] {
        let key = field.split('{').next().unwrap();
        let source = format!(
            "#+BEGIN: columnview :id global :format \"%ITEM %{field}\"\nprior\n#+END:\n* First\n:PROPERTIES:\n:{key}: 2\n:END:\n* {name}\n:PROPERTIES:\n:{key}: {value}\n:END:\n"
        );
        rejected(&source, OrgCommand::UpdateDynamicBlock, diagnostic);
    }
    let source = "#+COLUMNS: %ITEM\n#+COLUMNS: %EFFORT\n* Task\n";
    rejected(source, OrgCommand::Columns, "ambiguous");
}
// Real worker results cannot publish after source changes, C-g or a selected
// pane change. The independent source edit remains the only new history unit.
#[test]
fn column_request_guards() {
    let source = "#+BEGIN: columnview :id global :format \"%ITEM\"\nprior\n#+END:\n* Task\n";
    for mode in ["source", "cancel", "view"] {
        let (_fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", source);
        session.execute(Command::Split(rev::command::Split::Right));
        settle(&mut session);
        let selected = session.windows().selected_id();
        let other = session
            .windows()
            .iter()
            .find(|(id, _)| *id != selected)
            .unwrap()
            .0;
        session.execute(Command::Org(OrgCommand::UpdateDynamicBlock));
        match mode {
            "source" => {
                session.handle(Event::Paste("changed ".into()));
            }
            "cancel" => {
                session.handle(control('g'));
            }
            "view" => {
                session.select_view(other).unwrap();
            }
            _ => unreachable!(),
        }
        settle(&mut session);
        let expected = if mode == "source" {
            format!("changed {source}")
        } else {
            source.into()
        };
        assert_eq!(text(&session), expected, "{mode}: {}", session.echo());
        assert_eq!(
            session.buffer().unwrap().history().len(),
            usize::from(mode == "source"),
            "{mode}"
        );
        assert_eq!(session.remaining_input(Instant::now()), None, "{mode}");
    }
}
// Read-only canonical source may be projected but cannot authorize an explicit
// report update, including an otherwise valid prepared native column program.
#[test]
fn column_readonly() {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", "org.time.zone='UTC'"));
    let source = "#+BEGIN: columnview :id global :format \"%ITEM\"\nprior\n#+END:\n* Task\n";
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&fixture.write("notes.org", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    action(&mut session, OrgCommand::Columns);
    assert_eq!(session.query_agenda().unwrap().rows.len(), 1);
    session.handle(key(KeyCode::Char('q')));
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    assert!(session.echo().contains("read-only"), "{}", session.echo());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}
