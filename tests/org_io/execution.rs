use super::*;
use rev::org::execution::State;
fn editor(source: &str, extra: &str) -> (Fixture, Session) {
    let config = format!(
        "org.time.zone='UTC'\n{extra}\n[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\ncwd='source'\n"
    );
    configured(source, &config)
}
pub(super) fn configured(source: &str, config: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let snapshot = prepare(&f.write("config.toml", config));
    assert!(snapshot.diagnostic.is_none(), "{:?}", snapshot.diagnostic);
    let mut buffers = Buffers::with_settings(snapshot.snapshot).unwrap();
    let id = buffers.visit(&f.write("source.org", source)).unwrap();
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
// separate from old RESULTS and source bytes/history; owned HOME is removed.
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
    assert_eq!(text(&s), SOURCE);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert!(s.buffer().unwrap().history().is_empty());
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
