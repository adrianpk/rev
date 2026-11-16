use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Buffer, Buffers};
use rev::command::{Command, ModalCommand, Motion};
use rev::coordinates::Scalar;
use rev::display::{Frame, Size};
use rev::session::Session;
use rev::view::View;

use super::{now, observe::Observer};
use crate::fixture::Fixture;

pub const SOURCES: [(&str, &str); 3] = [
    ("plain", "Body target (outer [middle {leaf}]) tail\n"),
    (
        "org",
        "* Heading\nBody target (outer [middle {leaf}]) tail\n",
    ),
    (
        "markdown",
        "# Heading\nBody target (outer [middle {leaf}]) tail\n",
    ),
];

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub language: &'static str,
    pub profile: &'static str,
    pub style: &'static str,
}

impl Case {
    fn config(self) -> String {
        format!(
            "files.associations.'*.case'='{}'\neditor.interaction_profile='{}'\npairs.highlight='{}'\npairs.enabled=false\npairs.pulse_ms=250\neditor.soft_wrap=true\norg.indent=true\norg.headings.pretty=true\nmarkdown.pretty=true\ntree.visible=false\ngutter.visible=false\nbookmarks.markers=false\ncompletion.automatic=false\nlsp.enabled=false\npalette.foreground='#a1b2c3'\npalette.pairs.depth_colors=['#102030','#405060','#708090']\npalette.pairs.match_text='#abcdef'\npalette.pairs.match_background='#203040'\npalette.org.heading='#ca8e12'\npalette.markdown.heading='#ca8e12'",
            self.language, self.profile, self.style
        )
    }
}

pub struct Editor {
    _fixture: Fixture,
    pub session: Session,
    pub observer: Observer,
}

impl Editor {
    pub fn new(case: Case, source: &str, point: usize) -> Self {
        Self::prepared(case, source, point, |_, _| {})
    }

    pub fn prepared(
        case: Case,
        source: &str,
        point: usize,
        prepare: impl FnOnce(&mut Buffer, &mut View),
    ) -> Self {
        let fixture = Fixture::new();
        let startup = rev::config::prepare(&fixture.write("config.toml", &case.config()));
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        let id = buffers
            .visit(&fixture.write("source.case", source))
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        if case.language == "org" {
            let colors = rev::org::ColorScan::new(buffer, rev::org::Limits::default())
                .unwrap()
                .run(&rev::org::Cancellation::default());
            buffer.install_org_coloring(colors).unwrap();
        }
        let mut view = View::with_settings(buffer, &startup.snapshot);
        view.point.position = Scalar(point);
        prepare(buffer, &mut view);
        crate::pulse::reset();
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 120,
                height: 10,
            },
        )
        .unwrap()
        .with_pair_clock(now);
        if case.profile == "modal" {
            session.execute(Command::Modal(ModalCommand::Insert));
        }
        crossterm::style::force_color_output(std::env::var_os("NO_COLOR").is_none());
        Self {
            _fixture: fixture,
            session,
            observer: Observer::default(),
        }
    }

    pub fn handle(&mut self, code: KeyCode) {
        self.session
            .handle_at(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)), now());
    }

    pub fn source(&self) -> String {
        let b = self.session.buffer().unwrap();
        b.slice(&b.span(0..b.len_chars()).unwrap())
            .unwrap()
            .to_string()
    }

    pub fn settle(&mut self, mut check: impl FnMut(&Frame, &super::observe::Cells)) -> Frame {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                Instant::now() < deadline,
                "finite native continuity deadline"
            );
            let frame = self.session.poll_frame().unwrap();
            assert!(self.session.buffer().unwrap().retained_pair_bytes() <= 2097152);
            self.observer.record(&frame, &mut check);
            if self
                .session
                .remaining_input(now())
                .is_none_or(|wait| !wait.is_zero())
            {
                assert!(frame.cursor().is_some());
                return frame;
            }
            self.session.expire_input(now());
            std::thread::yield_now();
        }
    }

    pub fn opening(&mut self, source: &str) {
        self.session.execute(Command::Motion(Motion::BufferStart));
        for _ in 0..source.chars().position(|c| c == '(').unwrap() {
            self.session.execute(Command::Motion(Motion::ForwardChar));
        }
    }
}
