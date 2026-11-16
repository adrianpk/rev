use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::Command;
use rev::config::Palette;
use rev::coordinates::Scalar;
use rev::display::Frame;
use rev::session::Session;
use rev::view::{Size, View};

use crate::{fixture::Fixture, surface};

#[path = "rendering/colors.rs"]
mod colors;
#[path = "rendering/geometry.rs"]
mod geometry;
#[path = "rendering/overlays.rs"]
mod overlays;

pub(super) const SIZE: Size = Size {
    width: 70,
    height: 14,
};

pub(super) struct Editor {
    _fixture: Fixture,
    pub session: Session,
    pub palette: Palette,
}

impl Editor {
    pub fn new(config: &str, source: &str, point: usize) -> Self {
        Self::prepared(config, source, point, |_, _| {})
    }

    pub fn prepared(
        config: &str,
        source: &str,
        point: usize,
        prepare: impl FnOnce(&mut rev::buffer::Buffer, &mut View),
    ) -> Self {
        let fixture = Fixture::new();
        let startup = rev::config::prepare(&fixture.write(
            "config.toml",
            &format!(
                "gutter.visible=false\nbookmarks.markers=false\npairs.enabled=false\n{config}"
            ),
        ));
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        let id = buffers
            .visit(&fixture.write("source.case", source))
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        let mut view = View::with_settings(buffer, &startup.snapshot);
        view.point.position = Scalar(point);
        prepare(buffer, &mut view);
        let session = Session::new(buffers, view, SIZE).unwrap();
        Self {
            _fixture: fixture,
            session,
            palette: startup.snapshot.settings().palette.clone(),
        }
    }

    pub fn frame(&mut self) -> Frame {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                Instant::now() < deadline,
                "finite native rendering deadline"
            );
            self.session.expire_input(Instant::now());
            let frame = self.session.frame().unwrap();
            if self
                .session
                .remaining_input(Instant::now())
                .is_none_or(|wait| !wait.is_zero())
            {
                return frame;
            }
            std::thread::yield_now();
        }
    }

    pub fn choose(&mut self, style: &str) {
        self.session.execute(Command::PairHighlightStyle);
        self.session.handle(Event::Paste(style.into()));
        self.session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
    }

    pub fn unchanged(&self, source: &str, point: usize) {
        let buffer = self.session.buffer().unwrap();
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap()
                .to_string(),
            source
        );
        assert!(buffer.history().is_empty() && !buffer.is_modified() && buffer.is_readonly());
        assert_eq!(self.session.view().point.position, Scalar(point));
        assert_eq!((self.session.view().top, self.session.view().left), (0, 0));
    }
}

pub(super) fn marks(frame: &Frame) -> Vec<(usize, usize, usize)> {
    frame
        .rows()
        .iter()
        .enumerate()
        .flat_map(|(row, line)| {
            line.syntax().iter().filter_map(move |paint| {
                paint
                    .delimiter()
                    .map(|source| (source.0, row, usize::from(paint.columns.start)))
            })
        })
        .collect()
}

// Reuse the independent baseline and accepted delta oracle. Exact equality of
// every factory map and all 719 commands protects unrelated inherited controls.
#[test]
fn catalog_and_maps() {
    crate::accepted_maps::run().unwrap();
}
