use super::*;
use rev::org::execution::State;
fn editor(source: &str, extra: &str) -> (Fixture, Session) {
    let config = format!(
        "org.time.zone='UTC'\n{extra}\n[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\ncwd='source'\n"
    );
    configured(source, &config)
}
pub(super) fn configured(source: &str, config: &str) -> (Fixture, Session) {
    configured_readonly(source, config, false)
}
pub(super) fn configured_readonly(
    source: &str,
    config: &str,
    readonly: bool,
) -> (Fixture, Session) {
    let f = Fixture::new();
    let snapshot = prepare(&f.write("config.toml", config));
    assert!(snapshot.diagnostic.is_none(), "{:?}", snapshot.diagnostic);
    let mut buffers = Buffers::with_settings(snapshot.snapshot).unwrap();
    let id = buffers.visit(&f.write("source.org", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(readonly);
    let view = View::new(buffers.get(id).unwrap());
    let s = Session::new(
        buffers,
        view,
        Size {
            width: 110,
            height: 16,
        },
    )
    .unwrap();
    (f, s)
}
const SOURCE: &str = "#+BEGIN_SRC sh\nprintf started > marker\nprintf '%s' \"$HOME\" > home\nprintf 'hello 界'; printf 'warning' >&2\n#+END_SRC\n#+RESULTS:\n: previous\n";
// An actual selected shell reads staged stdin only after consent. Output stays
// in a checked RESULTS transaction; disk stays unchanged and owned HOME is removed.
#[test]
fn execute_selected() {
    let (f, mut s) = editor(SOURCE, "");
    let stamp = s.buffer().unwrap().stamp();
    action(&mut s, C::ExecuteSrc);
    assert!(!f.0.join("marker").exists());
    argument(&mut s, "maybe");
    assert!(!f.0.join("marker").exists());
    argument(&mut s, "yes");
    let out = s.execution_result().unwrap();
    assert_eq!(out.state, State::Succeeded, "{}", out.detail);
    assert_eq!(out.stdout, "hello 界".as_bytes());
    assert_eq!(out.stderr, b"warning");
    assert_eq!(out.status, Some(0));
    assert_eq!(text(&s), SOURCE.replace(": previous", ": hello 界"));
    assert_ne!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert_eq!(
        std::fs::read_to_string(f.0.join("source.org")).unwrap(),
        SOURCE
    );
    assert!(f.0.join("marker").exists());
    let home = std::fs::read_to_string(f.0.join("home")).unwrap();
    assert!(!Path::new(&home).exists(), "owned HOME survived cleanup");
    assert_eq!(s.remaining_input(Instant::now()), None);
    action(&mut s, C::ExecutionResults);
    argument(&mut s, "stdout");
    assert!(s.echo().contains("hello"));
}
// No/never restrictions remain effective even with confirmation disabled in
// configuration; :eval query restores the prompt. Decline starts no process.
#[test]
fn execute_admission() {
    for header in ["no", "never"] {
        let source = SOURCE.replace("BEGIN_SRC sh", &format!("BEGIN_SRC sh :eval {header}"));
        let (f, mut s) = editor(&source, "org.source.confirm_execution=false");
        action(&mut s, C::ExecuteSrc);
        assert!(s.echo().contains("forbids execution"), "{}", s.echo());
        assert!(s.execution_result().is_none());
        assert!(!f.0.join("marker").exists());
        assert_eq!(text(&s), source);
    }
    let source = SOURCE.replace("BEGIN_SRC sh", "BEGIN_SRC sh :eval query");
    let (f, mut s) = editor(&source, "org.source.confirm_execution=false");
    action(&mut s, C::CtrlCCtrlC);
    assert!(!f.0.join("marker").exists());
    argument(&mut s, "no");
    assert!(!f.0.join("marker").exists());
    assert_eq!(text(&s), source);
}

// Unsaved Unicode source is the job input; BOM/CRLF disk bytes remain exact.
// Read-only source can run after consent, with rejected publication and retained output.
#[test]
fn execute_source_fidelity() {
    let source =
        "\u{feff}#+BEGIN_SRC sh\r\nprintf original\r\n#+END_SRC\r\n#+RESULTS:\r\n: previous\r\n";
    let profile = "[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'";
    for readonly in [false, true] {
        let (f, mut s) = configured_readonly(source, profile, readonly);
        if !readonly {
            s.execute(Command::GotoLine);
            argument(&mut s, "2");
            s.handle(Event::Paste("printf unsaved; ".into()));
        }
        let before = text(&s);
        let stamp = s.buffer().unwrap().stamp();
        let history = s.buffer().unwrap().history().len();
        action(&mut s, C::ExecuteSrc);
        argument(&mut s, "yes");
        let out = s.execution_result().unwrap();
        assert_eq!(
            out.state,
            if readonly {
                State::Failed
            } else {
                State::Succeeded
            },
            "{}",
            out.detail
        );
        assert_eq!(
            out.stdout,
            if readonly {
                b"original".as_slice()
            } else {
                b"unsavedoriginal".as_slice()
            }
        );
        if readonly {
            assert_eq!(text(&s), before);
            assert_eq!(s.buffer().unwrap().stamp(), stamp);
            assert_eq!(s.buffer().unwrap().history().len(), history);
            assert!(out.detail.contains("read-only"));
        } else {
            assert_eq!(text(&s), before.replace(": previous", ": unsavedoriginal"));
            assert_ne!(s.buffer().unwrap().stamp(), stamp);
            assert_eq!(s.buffer().unwrap().history().len(), history + 1);
        }
        assert_eq!(s.buffer().unwrap().is_modified(), !readonly);
        assert_eq!(
            std::fs::read(f.0.join("source.org")).unwrap(),
            source.as_bytes()
        );
    }
}
// Several matching runners require explicit name selection, even when launch
// confirmation is disabled. Selecting cat proves code is not sent to a shell.
#[test]
fn execute_program_selection() {
    let profile = "org.source.confirm_execution=false\n[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\ncwd='source'\n[[org.source.programs]]\nname='reader'\nlanguage='sh'\nexecutable='/bin/cat'";
    let (f, mut s) = configured(SOURCE, profile);
    action(&mut s, C::ExecuteSrc);
    assert!(!f.0.join("marker").exists());
    argument(&mut s, "reader");
    let out = s.execution_result().unwrap();
    assert_eq!(out.state, State::Succeeded, "{}", out.detail);
    assert_eq!(out.program, "reader");
    assert!(out.stdout.starts_with(b"printf started"));
    assert!(!f.0.join("marker").exists());
    let result: String = out
        .stdout
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| format!(": {}\n", std::str::from_utf8(line).unwrap()))
        .collect();
    let prefix = SOURCE.split("#+RESULTS:").next().unwrap();
    assert_eq!(text(&s), format!("{prefix}#+RESULTS:\n{result}"));
}
