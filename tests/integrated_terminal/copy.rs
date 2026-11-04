use super::*;
use rev::command::{LinearCommand, Motion, RegionCommand};

fn copy(editor: &mut Editor) {
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::CopyMode));
    assert!(
        editor.session.terminal_copy_view().is_some(),
        "{}",
        editor.session.echo()
    );
}
fn select_alpha(editor: &mut Editor) {
    editor.session.execute(Command::Motion(Motion::BufferStart));
    editor.session.execute(Command::Region(RegionCommand::Set));
    editor.session.execute(Command::Motion(Motion::ForwardWord));
    editor.session.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(editor.session.killed_text().unwrap().to_string(), "alpha");
}

// A frozen native projection, region and search survive actual child output.
// Leaving copy mode sends no Return, and the source/terminal resource stays typed.
#[test]
fn stable_copy() {
    if isolated("panel::copy::stable_copy", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let script = "stty raw -echo\nprintf 'alpha 界é omega\\r\\nREADY'\nwhile [ ! -f burst ]; do sleep 0.01; done\ni=0; while [ $i -lt 100 ]; do printf '\\r\\nNEW%s' \"$i\"; i=$((i+1)); done\nprintf '\\r\\nOUTPUT_CONTINUES'\ndd bs=1 count=1 of=input.bin 2>/dev/null\nprintf RECEIVED\nwhile :; do sleep 60; done\n";
        let mut editor = Editor::with_script(profile, 24, "", script);
        let source = editor.session.buffer().unwrap().stamp();
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::Open));
        editor.wait(|s| text(s).contains("READY"));
        let id = editor
            .session
            .terminal_buffer()
            .unwrap()
            .terminal()
            .unwrap();
        let pid = editor.session.terminals().get(id).unwrap().process_id();
        editor
            .session
            .handle(key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        editor
            .session
            .handle(key(KeyCode::Char('t'), KeyModifiers::CONTROL));
        assert!(editor.session.terminal_copy_view().is_some());
        select_alpha(&mut editor);
        let point = editor.session.terminal_copy_view().unwrap().point;
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::MoveToBuffer));
        assert_eq!(editor.session.terminal_copy_view().unwrap().point, point);
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::MoveToPanel));
        assert_eq!(editor.session.terminal_copy_view().unwrap().point, point);
        editor.session.insert('z');
        assert!(
            editor.session.echo().contains("read-only"),
            "{}",
            editor.session.echo()
        );
        let before: Vec<_> = editor.session.frame().unwrap().rows()[12..22]
            .iter()
            .map(|r| r.text().to_owned())
            .collect();
        std::fs::write(editor.fixture.0.join("source/burst"), "ready").unwrap();
        editor.wait(|s| text(s).contains("OUTPUT_CONTINUES"));
        let after: Vec<_> = editor.session.frame().unwrap().rows()[12..22]
            .iter()
            .map(|r| r.text().to_owned())
            .collect();
        assert_eq!(after, before);
        assert_eq!(editor.session.terminal_copy_view().unwrap().point, point);
        editor.session.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(editor.session.killed_text().unwrap().to_string(), "alpha");
        editor.session.execute(Command::SearchForward);
        editor.session.handle(Event::Paste("omega".into()));
        assert!(editor.session.terminal_copy_view().unwrap().point.position > point.position);
        editor
            .session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert_eq!(editor.session.terminal_copy_view().unwrap().point, point);
        editor.session.execute(Command::SearchForward);
        editor.session.handle(Event::Paste("omega".into()));
        editor
            .session
            .handle(key(KeyCode::Enter, KeyModifiers::NONE));
        assert!(editor.session.terminal_copy_view().is_some());
        editor
            .session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert!(editor.session.terminal_copy_view().is_some());
        editor
            .session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert!(editor.session.terminal_copy_view().is_none());
        assert!(editor.session.terminal_focused());
        copy(&mut editor);
        editor
            .session
            .handle(key(KeyCode::Enter, KeyModifiers::NONE));
        assert!(editor.session.terminal_copy_view().is_none());
        assert_eq!(
            std::fs::metadata(editor.fixture.0.join("source/input.bin"))
                .unwrap()
                .len(),
            0
        );
        editor
            .session
            .handle(key(KeyCode::Char('p'), KeyModifiers::NONE));
        editor.wait(|s| text(s).contains("RECEIVED"));
        assert_eq!(
            std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
            b"p"
        );
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        assert_eq!(editor.session.terminal_buffer().unwrap().len_bytes(), 0);
        assert_eq!(
            editor.session.terminals().get(id).unwrap().process_id(),
            pid
        );
    }
}

// Two attachments freeze at different times without cloning the child or
// replacing the terminal-backed buffer. Their native points remain independent.
#[test]
fn copy_views() {
    if isolated("panel::copy::copy_views", false) {
        return;
    }
    let script = "printf 'FIRST\\nREADY'\nwhile [ ! -f burst ]; do sleep 0.01; done\nprintf '\\033[2J\\033[HSECOND\\nREADY2'\nwhile :; do sleep 60; done\n";
    let mut editor = Editor::with_script("modal", 24, "", script);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::NewBuffer));
    editor.wait(|s| text(s).contains("READY"));
    let id = editor.session.buffer().unwrap().terminal().unwrap();
    let pid = editor.session.terminals().get(id).unwrap().process_id();
    copy(&mut editor);
    editor.session.execute(Command::Motion(Motion::BufferStart));
    let first = editor.session.terminal_copy_view().unwrap().point;
    editor
        .session
        .execute(Command::Split(rev::window::Split::Right));
    let views = editor.session.windows().order();
    editor.session.select_view(views[1]).unwrap();
    assert!(editor.session.terminal_copy_view().is_none());
    std::fs::write(editor.fixture.0.join("source/burst"), "ready").unwrap();
    editor.wait(|s| text(s).contains("READY2"));
    copy(&mut editor);
    editor.session.execute(Command::Motion(Motion::BufferStart));
    // Screen clearing retains older rows; locate the newer marker explicitly.
    editor.session.execute(Command::SearchForward);
    editor.session.handle(Event::Paste("SECOND".into()));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
    editor
        .session
        .execute(Command::Navigate(rev::command::Navigation::new(
            rev::command::navigation::Target::Align(rev::command::navigation::Screen::Top),
        )));
    let second = editor.session.terminal_copy_view().unwrap().point;
    editor.session.select_view(views[0]).unwrap();
    assert_eq!(editor.session.terminal_copy_view().unwrap().point, first);
    let frame = editor.session.frame().unwrap();
    assert!(frame.rows()[0].text().contains("FIRST"));
    assert!(frame.rows()[0].text().contains("SECOND"));
    editor.session.select_view(views[1]).unwrap();
    assert_eq!(editor.session.terminal_copy_view().unwrap().point, second);
    assert_eq!(editor.session.terminals().ids().count(), 1);
    assert_eq!(
        editor.session.terminals().get(id).unwrap().process_id(),
        pid
    );
}

// The text materialization cap rejects a legal grid with retained combining
// extras. Rejection preserves the live process and typed source resource.
#[test]
fn copy_admission() {
    if isolated("panel::copy::copy_admission", false) {
        return;
    }
    let cell = format!("a{}", "\u{301}".repeat(32));
    let script = format!(
        "i=0; while [ $i -lt 16800 ]; do printf '%s' '{cell}'; i=$((i+1)); done\nprintf '\\r\\nREADY'\nwhile :; do sleep 60; done\n"
    );
    let mut editor = Editor::with_script("non-modal", 24, "", &script);
    let source = editor.session.buffer().unwrap().stamp();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    editor.wait(|s| text(s).contains("READY"));
    let id = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    let pid = editor.session.terminals().get(id).unwrap().process_id();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::CopyMode));
    assert!(
        editor.session.echo().contains("exceeds 1 MiB"),
        "{}",
        editor.session.echo()
    );
    assert!(editor.session.terminal_copy_view().is_none());
    assert!(editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert_eq!(
        editor.session.terminals().get(id).unwrap().process_id(),
        pid
    );
}

fn strokes(editor: &mut Editor, text: &str) {
    for character in text.chars() {
        editor
            .session
            .handle(key(KeyCode::Char(character), KeyModifiers::NONE));
    }
}

// Real profile bindings resolve operators and regions against the private copy,
// including inclusive blocks and complete wide/combining grapheme transfers.
#[test]
fn native_copy() {
    if isolated("panel::copy::native_copy", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let script =
            "printf 'alpha beta\\nB123456l9l\\n界é omega\\nREADY'\nwhile :; do sleep 60; done\n";
        let mut editor = Editor::with_script(profile, 24, "", script);
        editor.open_copy();
        if profile == "modal" {
            strokes(&mut editor, "ggyw");
            assert_eq!(editor.session.killed_text().unwrap().to_string(), "alpha ");
            assert_eq!(
                editor
                    .session
                    .terminal_copy_view()
                    .unwrap()
                    .point
                    .position
                    .0,
                0
            );
            strokes(&mut editor, "yy");
            let line = editor.session.killed_text().unwrap().to_string();
            assert!(line.starts_with("alpha beta"));
            assert!(line.ends_with('\n'));
            strokes(&mut editor, "jfl;");
            assert_eq!(
                editor
                    .session
                    .terminal_copy_view()
                    .unwrap()
                    .point
                    .position
                    .0,
                90
            );
            strokes(&mut editor, "ggvlly");
            assert_eq!(editor.session.killed_text().unwrap().to_string(), "alp");
            editor
                .session
                .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
            strokes(&mut editor, "gg");
            editor
                .session
                .handle(key(KeyCode::Char('v'), KeyModifiers::CONTROL));
            strokes(&mut editor, "jly");
            assert_eq!(editor.session.killed_text().unwrap().to_string(), "al\nB1");
        } else {
            editor.session.execute(Command::Motion(Motion::BufferStart));
            editor
                .session
                .handle(key(KeyCode::Char(' '), KeyModifiers::CONTROL));
            editor
                .session
                .handle(key(KeyCode::Char('f'), KeyModifiers::ALT));
            editor
                .session
                .handle(key(KeyCode::Char('w'), KeyModifiers::ALT));
            assert_eq!(editor.session.killed_text().unwrap().to_string(), "alpha");
            editor
                .session
                .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
            editor.session.execute(Command::Motion(Motion::BufferStart));
            editor
                .session
                .execute(Command::Rectangle(rev::command::RectangleCommand::Mark));
            editor
                .session
                .handle(key(KeyCode::Right, KeyModifiers::NONE));
            editor
                .session
                .handle(key(KeyCode::Right, KeyModifiers::NONE));
            editor
                .session
                .handle(key(KeyCode::Down, KeyModifiers::NONE));
            editor
                .session
                .execute(Command::Rectangle(rev::command::RectangleCommand::Copy));
            assert_eq!(editor.session.rectangle().unwrap().rows(), &["al", "B1"]);
        }
        editor
            .session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert!(editor.session.terminal_copy_view().is_some());
        editor.session.execute(Command::Motion(Motion::BufferStart));
        editor
            .session
            .execute(Command::Scroll(rev::command::Scroll::NextLine));
        editor
            .session
            .execute(Command::Scroll(rev::command::Scroll::NextLine));
        editor.session.execute(Command::Region(RegionCommand::Set));
        editor.session.execute(Command::Motion(Motion::ForwardChar));
        editor.session.execute(Command::Motion(Motion::ForwardChar));
        editor.session.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(editor.session.killed_text().unwrap().to_string(), "界é");
        assert_eq!(editor.session.terminal_buffer().unwrap().len_bytes(), 0);
        assert_eq!(editor.session.terminals().ids().count(), 1);
    }
}

impl Editor {
    fn open_copy(&mut self) {
        self.session
            .execute(Command::Terminal(TerminalCommand::Open));
        self.wait(|s| text(s).contains("READY"));
        copy(self);
    }
}
