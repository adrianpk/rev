use super::*;
use rev::command::BufferCommand;

fn argument(s: &mut Session, value: &str) {
    s.handle(Event::Paste(value.into()));
    s.handle(key(KeyCode::Enter));
}
fn stopped(s: &mut Session) {
    support::until(|| {
        tick(s);
        s.remaining_input(Instant::now()).is_none()
    });
}
fn proposal(s: &mut Session) {
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
fn models(s: &mut Session) {
    support::until(|| {
        tick(s);
        assert!(
            s.remaining_input(Instant::now()).is_some()
                || s.echo().contains("Choose a Pi-reported"),
            "{}",
            s.echo()
        );
        s.echo().contains("Choose a Pi-reported")
    });
}
fn clear(s: &mut Session, count: usize) {
    s.handle(key(KeyCode::Home));
    for _ in 0..count {
        s.handle(key(KeyCode::Delete));
    }
}

// Both command surfaces choose a session-local root and a Pi-reported exact
// model. Generation retains its original source target and leaves config alone.
#[test]
fn native_controls() {
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let mut s = editor(&f, "normal", profile, false);
        let config = fs::read(f.0.join("config.toml")).unwrap();
        let source = s.buffer().unwrap().stamp();
        let root = f.0.join("selected");
        fs::create_dir(&root).unwrap();
        named(&mut s, profile, "ai-set-directory");
        clear(&mut s, f.0.to_str().unwrap().chars().count());
        argument(&mut s, root.to_str().unwrap());
        assert!(s.echo().contains("AI directory selected"), "{}", s.echo());
        named(&mut s, profile, "ai-select-model");
        models(&mut s);
        argument(&mut s, "another/model");
        assert!(s.echo().contains("AI model selected"), "{}", s.echo());
        named(&mut s, profile, "ai-implement");
        argument(&mut s, "Insert only at the captured point.");
        proposal(&mut s);
        let args: Vec<String> =
            serde_json::from_str(&fs::read_to_string(f.0.join("args")).unwrap()).unwrap();
        let model = args.iter().position(|arg| arg == "--model").unwrap();
        assert_eq!(args[model + 1], "another/model");
        assert_eq!(
            fs::read_to_string(f.0.join("cwd")).unwrap(),
            root.to_str().unwrap()
        );
        assert!(text(&s).contains(&format!("Directory root: {}", root.display())));
        s.execute(Command::AiDiscardResult);
        assert_eq!(s.buffer().unwrap().stamp(), source);
        assert_eq!(fs::read(f.0.join("config.toml")).unwrap(), config);
    }
}

// An idle connection owns no editor input timer. Model queries reuse that child;
// changing the model stops it before a fresh generation and reset is scoped.
#[test]
fn idle_and_disconnect() {
    let f = Fixture::new();
    let mut s = editor(&f, "idle", "non-modal", false);
    s.execute(Command::AiPrompt);
    argument(&mut s, "Generate a proposal.");
    proposal(&mut s);
    assert!(s.remaining_input(Instant::now()).is_none());
    let pid = fs::read_to_string(f.0.join("pid")).unwrap();
    s.execute(Command::AiDiscardResult);
    s.execute(Command::AiSelectModel);
    models(&mut s);
    assert_eq!(fs::read_to_string(f.0.join("pid")).unwrap(), pid);
    argument(&mut s, "another/model");
    stopped(&mut s);
    assert!(!std::path::Path::new("/proc").join(&pid).exists());
    s.execute(Command::AiPrompt);
    argument(&mut s, "Generate with the selected model.");
    proposal(&mut s);
    s.execute(Command::AiDiscardResult);
    s.execute(Command::AiResetSession);
    stopped(&mut s);
    s.execute(Command::AiPrompt);
    argument(&mut s, "Start fresh.");
    proposal(&mut s);
    s.execute(Command::AiDiscardResult);
    s.execute(Command::AiDisconnect);
    stopped(&mut s);
    let pid = fs::read_to_string(f.0.join("pid")).unwrap();
    assert!(!std::path::Path::new("/proc").join(pid).exists());
    assert_eq!(text(&s), "é original selection\n");
}

// Cancellation owns an active model lookup or its native chooser. Catalog
// completion never steals a different prompt and an unreported model rejects.
#[test]
fn model_input_ownership() {
    let f = Fixture::new();
    let mut s = editor(&f, "normal", "non-modal", false);
    let original = s.buffer().unwrap().stamp();
    s.execute(Command::AiSelectModel);
    models(&mut s);
    argument(&mut s, "not-reported/model");
    assert!(s.echo().contains("Choose exactly"));
    assert!(!f.0.join("prompt").exists());
    s.execute(Command::AiSelectModel);
    models(&mut s);
    s.execute(Command::KeyboardQuit);
    assert_eq!(s.buffer().unwrap().stamp(), original);
    s.execute(Command::AiSelectModel);
    s.execute(Command::Buffer(BufferCommand::Switch));
    support::until(|| {
        tick(&mut s);
        s.remaining_input(Instant::now()).is_none()
    });
    assert!(
        s.echo().contains("another input owns focus"),
        "{}",
        s.echo()
    );
    s.execute(Command::KeyboardQuit);
    assert_eq!(s.buffer().unwrap().stamp(), original);
}
