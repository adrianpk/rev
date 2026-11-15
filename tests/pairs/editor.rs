use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::session::Session;
use rev::view::{Size, View};

use crate::fixture::Fixture;

pub(super) struct Editor {
    pub(super) session: Session,
    _fixture: Fixture,
}

impl Editor {
    pub(super) fn new(config: &str, language: &str, marked: &str) -> Self {
        Self::build(config, language, marked, false)
    }

    pub(super) fn build(config: &str, language: &str, marked: &str, readonly: bool) -> Self {
        let fixture = Fixture::new();
        let config =
            format!("syntax.enabled=false\nfiles.associations.'*.case'='{language}'\n{config}");
        let startup = rev::config::prepare(&fixture.write("config.toml", &config));
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let byte = marked.find('|').expect("one point marker");
        let point = marked[..byte].chars().count();
        let source = marked.replacen('|', "", 1);
        let id = buffers
            .visit(&fixture.write("source.case", &source))
            .unwrap();
        buffers.get_mut(id).unwrap().set_readonly(readonly);
        let mut view = View::new(buffers.get(id).unwrap());
        view.point.position = Scalar(point);
        Self {
            session: Session::new(
                buffers,
                view,
                Size {
                    width: 80,
                    height: 24,
                },
            )
            .unwrap(),
            _fixture: fixture,
        }
    }

    pub(super) fn text(&self) -> String {
        let buffer = self.session.buffer().unwrap();
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string()
    }

    pub(super) fn expect(&self, marked: &str) {
        let byte = marked.find('|').unwrap();
        assert_eq!(self.text(), marked.replacen('|', "", 1));
        assert_eq!(
            self.session.view().point.position.0,
            marked[..byte].chars().count()
        );
    }

    pub(super) fn type_key(&mut self, c: char) {
        self.session.handle(character(c));
        settle(&mut self.session);
    }

    pub(super) fn counted(&mut self, count: &str, c: char) {
        self.session
            .handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
        for digit in count.chars() {
            self.session.handle(character(digit));
        }
        self.type_key(c);
    }
}

pub(super) fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}
pub(super) fn character(c: char) -> Event {
    key(KeyCode::Char(c), KeyModifiers::NONE)
}
pub(super) fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while session
        .remaining_input(Instant::now())
        .is_some_and(|wait| wait == Duration::ZERO)
    {
        assert!(Instant::now() < deadline, "pair typing did not settle");
        session.expire_input(Instant::now());
    }
}
