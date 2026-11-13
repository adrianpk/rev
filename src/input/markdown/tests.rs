use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::command::{Command, CompletionCommand, MarkdownCommand};
use crate::config::prepare;
use crate::input::{EditingState, Input, Outcome};

// Popup acceptance owns Return before either document map. Effective selector
// hints come from the originating profile, then retire when its source changes.
#[test]
fn popup_and_hints() {
    for (profile, state) in [("non-modal", None), ("modal", Some(EditingState::Insert))] {
        let fixture = crate::fixture::Fixture::new();
        let path = fixture.write(
            "config.toml",
            &format!("editor.interaction_profile='{profile}'"),
        );
        let startup = prepare(&path);
        assert!(startup.diagnostic.is_none());
        let mut input = Input::with_settings(&startup.snapshot);
        if let Some(state) = state {
            input.set_editing_state(state);
        }
        input.set_markdown_context(true);
        input.commands().unwrap();
        assert_eq!(
            input
                .palette()
                .unwrap()
                .hint(Command::Markdown(MarkdownCommand::Return)),
            Some("RET"),
            "{profile}"
        );
        input.set_markdown_context(false);
        assert_eq!(
            input
                .palette()
                .unwrap()
                .hint(Command::Markdown(MarkdownCommand::Return)),
            None
        );
        input.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        input.set_markdown_context(true);
        input.set_markdown_table(true);
        input.commands().unwrap();
        assert_eq!(
            input
                .palette()
                .unwrap()
                .hint(Command::Markdown(MarkdownCommand::TableNextField)),
            Some("TAB"),
            "{profile}"
        );
        input.set_markdown_table(false);
        assert_eq!(
            input
                .palette()
                .unwrap()
                .hint(Command::Markdown(MarkdownCommand::TableNextField)),
            None,
            "{profile}"
        );
        input.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        input.set_markdown_table(true);
        input.set_completion(true);
        assert_eq!(
            input.handle(key(KeyCode::Enter, KeyModifiers::NONE)),
            Outcome::Command(Command::Completion(CompletionCommand::Accept)),
            "{profile}"
        );
        assert_eq!(
            input.handle(key(KeyCode::Tab, KeyModifiers::NONE)),
            Outcome::Command(Command::Completion(CompletionCommand::Accept)),
            "{profile}"
        );
        input.set_completion(false);
        assert_eq!(
            input.handle(key(KeyCode::Enter, KeyModifiers::NONE)),
            Outcome::Command(Command::Markdown(MarkdownCommand::Return)),
            "{profile}"
        );
    }
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}
