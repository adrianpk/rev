use super::*;
use rev::command::BufferCommand;

fn argument(s: &mut Session, value: &str) {
    s.handle(Event::Paste(value.into()));
    s.handle(key(KeyCode::Enter));
}
fn switch(s: &mut Session, name: &str) {
    s.execute(Command::Buffer(BufferCommand::Switch));
    argument(s, name);
}
fn prompt(s: &mut Session) {
    s.execute(Command::AiPrompt);
    argument(s, "Use only admitted source observations.");
    support::until(|| {
        tick(s);
        assert!(
            s.remaining_input(Instant::now()).is_some() || s.echo().contains("proposal ready"),
            "{}",
            s.echo()
        );
        s.echo().contains("proposal ready")
    });
}

// Both the current clean source and hidden dirty peers use complete native
// revisions. A new invocation refreshes the map; disk changes never substitute
// for unsaved text, and directory context does not grant another edit target.
#[test]
fn native_source_map() {
    let f = Fixture::new();
    let mut s = editor(&f, "normal", "non-modal", false);
    let other = f.write("module.zig", "disk only\n");
    s.execute(Command::FindFile);
    argument(&mut s, other.to_str().unwrap());
    s.insert('é');
    let first = s.buffer().unwrap().stamp();
    switch(&mut s, "source.txt");
    fs::write(&other, "changed outside\n").unwrap();
    prompt(&mut s);
    assert!(text(&s).contains(&format!("Directory root: {}", f.0.display())));
    let message: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(f.0.join("prompt")).unwrap()).unwrap();
    assert_eq!(message["context"], "directory");
    let sources = message["dirty_sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2);
    let observed = sources.iter().find(|v| v["path"] == "module.zig").unwrap();
    assert_eq!(observed["text"], "édisk only\n");
    assert_eq!(observed["revision"], first.revision.to_string());
    assert_eq!(observed["dirty"], true);
    assert_eq!(observed["complete"], true);
    s.execute(Command::AiDiscardResult);
    switch(&mut s, "module.zig");
    // Buffer switching retains the prior point; this observation prepends text.
    s.execute(Command::Motion(Motion::BufferStart));
    s.insert('x');
    let current = s.buffer().unwrap().stamp();
    switch(&mut s, "source.txt");
    prompt(&mut s);
    let message: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(f.0.join("prompt")).unwrap()).unwrap();
    let observed = message["dirty_sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "module.zig")
        .unwrap();
    assert_eq!(observed["text"], "xédisk only\n");
    assert_eq!(observed["revision"], current.revision.to_string());
    assert_eq!(fs::read_to_string(other).unwrap(), "changed outside\n");
}

// Excluded credential snapshots and an overfull dirty map fail before any
// process. No silent disk fallback or partial source-map admission is allowed.
#[test]
fn source_map_admission() {
    for case in ["credentials", "count"] {
        let f = Fixture::new();
        let mut s = editor(&f, "normal", "non-modal", false);
        for n in 0..if case == "count" { 16 } else { 1 } {
            let path = f.write(
                &if case == "count" {
                    format!("peer-{n}.zig")
                } else {
                    ".env".into()
                },
                "private source",
            );
            s.execute(Command::FindFile);
            argument(&mut s, path.to_str().unwrap());
            s.insert('x');
        }
        switch(&mut s, "source.txt");
        let source = s.buffer().unwrap().stamp();
        s.execute(Command::AiPrompt);
        argument(&mut s, "Generate a proposal.");
        assert!(
            s.echo().contains(if case == "count" {
                "16 buffers"
            } else {
                "excluded"
            }),
            "{}",
            s.echo()
        );
        assert_eq!(s.buffer().unwrap().stamp(), source);
        assert!(!f.0.join("version-pid").exists());
    }
}
