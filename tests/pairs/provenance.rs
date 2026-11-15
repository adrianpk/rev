#[path = "provenance/literals.rs"]
mod literals;
#[path = "provenance/macros.rs"]
mod macros;
#[path = "provenance/projections.rs"]
mod projections;

use super::editor::*;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, History, ModalCommand, Motion};
use rev::session::Session;
use std::time::{Duration, Instant};

fn mode(editor: &mut Editor, profile: &str) {
    if profile == "modal" {
        editor.session.execute(Command::Modal(ModalCommand::Insert));
    }
}

fn function(session: &mut Session, number: u8) {
    session.handle(key(KeyCode::F(number), KeyModifiers::NONE));
    settle(session);
}

fn observe(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native observation did not settle"
        );
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
